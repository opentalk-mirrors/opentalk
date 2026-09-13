// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use anyhow::{Context, bail};
use redis::{AsyncConnectionConfig, PushInfo, aio::MultiplexedConnection};
use tokio::sync::mpsc;
use uuid::Uuid;

use crate::{
    storage::redis::{
        KEEP_ALIVE_INTERVAL, KEEP_ALIVE_TTL, scripts,
        util::{PMessageData, redis_to_string},
    },
    tasks::ShutdownReceiver,
};

/// The Redis keyspace notification subscription pattern for orchestrator keep-alive keys
///
/// This pattern matches any event for keys of the form `ot-orchestrator:orchestrator:<uuid>:alive`
/// in any database.
///
/// `__keyspace`: Listen for events on keys (as opposed to `__keyevent` which listens for events on
/// values). `@*__`: The `@*` indicates that the event can match any redis database.
/// `ot-orchestrator:orchestrator:*:alive`: The key pattern that needs to match, where * is the
/// orchestrator UUID.
const ORCHESTRATOR_ALIVE_KEY_SUBSCRIPTION: &str =
    "__keyspace@*__:ot-orchestrator:orchestrator:*:alive";

/// The PeerMonitor monitors the health of other orchestrator instances in a
/// cluster deployment.
///
/// When another orchestrator instance fails to refresh their keep-alive key in redis, the
/// PeerMonitor will clean up related resources for that orchestrator instance.
#[derive(Debug)]
pub struct PeerMonitor {
    /// The redis client
    client: redis::Client,
    /// The redis connection that we are using to receive keyspace notifications
    redis_conn: MultiplexedConnection,
    /// The redis database number that we are connected to (used for keyspace notifications)
    redis_db: i64,
    /// The receiver for push messages from the redis connection
    push_receiver: mpsc::UnboundedReceiver<PushInfo>,
    /// The orchestrator id of this instance
    id: Uuid,
}

impl PeerMonitor {
    /// Initializes a new PeerMonitor
    ///
    /// Connects to redis and sets up keyspace notifications for orchestrator keep-alive keys.
    pub async fn init(client: redis::Client, id: Uuid) -> anyhow::Result<Self> {
        let (con, push_receiver) = setup_redis_connection(&client).await?;
        let redis_db = client.get_connection_info().redis_settings().db();

        Ok(Self {
            client,
            redis_conn: con,
            redis_db,
            push_receiver,
            id,
        })
    }

    /// Run the peer monitor loop
    pub async fn run(mut self, shutdown_rx: ShutdownReceiver) -> anyhow::Result<()> {
        loop {
            match self.inner_run(shutdown_rx.resubscribe()).await {
                Ok(_) => return Ok(()),
                Err(e) => {
                    tracing::error!("Peer monitor task exited with error: {e:?}");
                    tracing::info!("Attempting to reconnect to Redis...");
                    tokio::time::sleep(std::time::Duration::from_secs(2)).await;

                    // try to reconnect once before exiting
                    match setup_redis_connection(&self.client).await {
                        Ok((con, rx)) => {
                            self.redis_conn = con;
                            self.push_receiver = rx;
                            tracing::info!("Peer monitor reconnected to Redis");
                            continue;
                        }
                        Err(e) => {
                            tracing::error!("Failed to reconnect peer monitor to Redis: {e}");
                        }
                    }

                    return Err(e.context("Peer monitor task exited with error"));
                }
            }
        }
    }

    async fn inner_run(&mut self, mut shutdown_rx: ShutdownReceiver) -> anyhow::Result<()> {
        let mut keep_alive_interval = tokio::time::interval(KEEP_ALIVE_INTERVAL);

        loop {
            tokio::select! {
                Some(push_info) = self.push_receiver.recv() => {
                    self.handle_redis_notification(push_info).await?;
                }
                _ = keep_alive_interval.tick() => {
                    redis::cmd("SET")
                        .arg(format!("ot-orchestrator:orchestrator:{}:alive", self.id))
                        .arg(self.id.to_string())
                        .arg("EX")
                        .arg(KEEP_ALIVE_TTL)
                        .query_async::<()>(&mut self.redis_conn)
                        .await
                        .context("Failed to send keep-alive to Redis")?;
                }
                _ = shutdown_rx.wait_for_shutdown() => {
                    tracing::info!("Peer monitor received shutdown signal, exiting");
                    return Ok(());
                }
            }
        }
    }

    /// Handles incoming Redis keyspace notifications
    ///
    /// This currently only listens for notifications for the keep-alive key of orchestrators,
    /// removing them and associated resources when their keep-alive key expires.
    async fn handle_redis_notification(&mut self, push_info: PushInfo) -> anyhow::Result<()> {
        match push_info.kind {
            redis::PushKind::PMessage => {
                let pmessage = PMessageData::try_from(push_info.data)
                    .context("Failed to parse PMessageData")?;

                tracing::trace!("received pmessage from redis: {}", pmessage);

                self.handle_pmessage(pmessage).await?;
            }
            redis::PushKind::PSubscribe => {
                // get the channel name
                let Some(data) = push_info.data.first() else {
                    tracing::error!("missing channel name in PSubscribe event");
                    return Ok(());
                };
                let Ok(str) = redis_to_string(data) else {
                    tracing::warn!(
                        "Unexpected Redis push notification data: {:?}",
                        push_info.data
                    );
                    return Ok(());
                };

                tracing::debug!("Subscribed to Redis channel: {str}");
            }
            _ => {
                tracing::warn!(
                    "Received unexpected Redis push notification: {:?}",
                    push_info
                );
            }
        }

        Ok(())
    }

    async fn handle_pmessage(
        &self,
        PMessageData {
            pattern,
            channel,
            payload,
        }: PMessageData,
    ) -> anyhow::Result<()> {
        if pattern != ORCHESTRATOR_ALIVE_KEY_SUBSCRIPTION {
            tracing::debug!(
                "Received unexpected Redis push notification pattern: {}",
                pattern
            );
            return Ok(());
        }

        if payload != "expired" {
            tracing::debug!(
                "Received unexpected Redis push notification payload: {}",
                payload
            );
            return Ok(());
        }

        let Some(orchestrator_id) = self
            .parse_alive_key_event(channel)
            .context("Failed to parse alive event channel")?
        else {
            // not addressed to this database, ignore
            return Ok(());
        };

        if orchestrator_id == self.id {
            bail!("Received key expiration for own orchestrator id: {orchestrator_id}");
        }

        tracing::debug!("Orchestrator {orchestrator_id} timed out (keep-alive key expired)");

        scripts::cleanup(&self.client, &[orchestrator_id])
            .await
            .context("Failed to cleanup timed out orchestrator")?;

        Ok(())
    }

    /// Parses a Redis keyspace notification channel for an alive key event and returns the
    /// orchestrator ID
    ///
    /// The channel is expected to be in the format:
    /// `__keyspace@0__:ot-orchestrator:orchestrator:e13127e7-899a-4dcc-993a-28e5bc15cb11:alive`
    fn parse_alive_key_event(&self, channel: String) -> anyhow::Result<Option<Uuid>> {
        // Extract db index between @ and __
        let after_at = channel
            .strip_prefix("__keyspace@")
            .context("Failed to parse channel prefix")?;

        let (db_str, rest) = after_at
            .split_once("__:")
            .context("Failed to parse redis database")?;

        let db: i64 = db_str.parse().context("Failed to parse")?;

        if db != self.redis_db {
            return Ok(None); // different database, ignore
        }

        let uuid_str = rest
            .strip_prefix("ot-orchestrator:orchestrator:")
            .context("Unexpected key prefix")?
            .strip_suffix(":alive")
            .context("Unexpected key suffix")?;

        Ok(Some(Uuid::parse_str(uuid_str).context(
            "Failed to parse uuid from alive key event channel",
        )?))
    }
}

/// Connect to redis and setup keyspace notifications
///
/// Returns a tuple of the redis connection and a receiver for push messages from redis
async fn setup_redis_connection(
    client: &redis::Client,
) -> anyhow::Result<(MultiplexedConnection, mpsc::UnboundedReceiver<PushInfo>)> {
    let (push_sender, push_receiver) = mpsc::unbounded_channel::<PushInfo>();
    let config = AsyncConnectionConfig::new().set_push_sender(push_sender);

    let mut con = client
        .get_multiplexed_async_connection_with_config(&config)
        .await
        .context("Unable to connect to redis")?;

    enable_redis_notifications(&mut con)
        .await
        .context("Failed to enable keyspace notifications")?;

    // receive keyspace notification for 'expired' event on the keep-alive key for orchestrators
    redis::cmd("PSUBSCRIBE")
        .arg(ORCHESTRATOR_ALIVE_KEY_SUBSCRIPTION)
        .query_async::<()>(&mut con)
        .await?;

    Ok((con, push_receiver))
}

/// Set up Redis keyspace notifications to get notified when an orchestrators keep-alive key
/// expires
async fn enable_redis_notifications(con: &mut MultiplexedConnection) -> anyhow::Result<()> {
    let (config_key, config_value) = redis::cmd("CONFIG")
        .arg("GET")
        .arg("notify-keyspace-events")
        .query_async::<redis::Value>(con)
        .await
        .context("Failed to read keyspace config")?
        .into_map_iter()
        .map_err(|other| anyhow::anyhow!("Expected config map, got {other:?}"))?
        .next()
        .unwrap_or_default();

    debug_assert!(config_key == redis::Value::BulkString(b"notify-keyspace-events".to_vec()));

    let mut keyspace_config = redis_to_string(&config_value).unwrap_or_default();

    if keyspace_config.contains("K") && keyspace_config.contains("x") {
        tracing::debug!("Redis keyspace notifications already enabled, nothing to do");
        return Ok(());
    };

    if !keyspace_config.contains("K") {
        tracing::info!("Redis keyspace notifications not enabled, enabling now",);

        keyspace_config.push('K');
    }

    if !keyspace_config.contains("x") {
        tracing::info!("Redis keyspace notifications for expired events not enabled, enabling now",);

        keyspace_config.push('x');
    }

    tracing::info!(
        "Setting Redis keyspace notifications config to: {}",
        keyspace_config
    );

    // enable redis keyspace events
    redis::cmd("CONFIG")
        .arg("SET")
        .arg("notify-keyspace-events")
        .arg(keyspace_config)
        .query_async::<()>(con)
        .await
        .context("Failed to enable keyspace notifications")?;

    Ok(())
}
