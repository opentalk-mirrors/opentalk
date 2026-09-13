// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use std::time::Duration;

use etcd_client::{Client, Compare, CompareOp, EventType, PutOptions, TxnOp, WatchResponse};
use snafu::{ResultExt, Snafu};
use tokio::{
    sync::{
        mpsc::{self, error::SendError},
        watch::{self, Ref},
    },
    time::interval,
};

use super::ETCD_LEASE_TTL;
use crate::error::{
    AddSnafu, ConnectSnafu, CreateKeepAliveSnafu, CreateWatchSnafu, EtcdError, KeepAliveSnafu,
    LeaseSnafu, RemoveSnafu, WatchProgressSnafu, WatchStreamSnafu,
};

const ELECTION_KEY: &[u8] = "opentalk/job_executor".as_bytes();

#[derive(Debug, Snafu)]
pub enum ElectionTaskError {
    /// The election task exited for some reason
    #[snafu(display("Election Task exited: {source}"))]
    ElectionTaskExitedError { source: SendError<()> },

    /// The etcd API returned an error
    #[snafu(transparent)]
    EtcdError {
        #[snafu(source(from(EtcdError, Box::new)))]
        source: Box<EtcdError>,
    },
}

/// State of the election task
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ElectionState {
    Follower,
    Leader,
    /// The initial state,
    Hold,
}

/// An etcd lease
struct Lease {
    /// The lease id
    id: i64,
    /// The leases time to live, the server may respond with a different TTL than requested by the client
    ttl: i64,
}

impl Lease {
    async fn new(client: &mut Client) -> Result<Self, ElectionTaskError> {
        let lease_response = client
            .lease_grant(ETCD_LEASE_TTL as i64, None)
            .await
            .context(LeaseSnafu)?;

        let lease_response_ttl = lease_response.ttl();

        if lease_response_ttl != ETCD_LEASE_TTL as i64 {
            log::warn!(
                "Requested lease with {ETCD_LEASE_TTL} seconds ttl, server responded with {lease_response_ttl} seconds ttl"
            );
        };

        Ok(Self {
            id: lease_response.id(),
            ttl: lease_response_ttl,
        })
    }
}

/// The [`ElectionTask`] manages the leader election between multiple [`JobRunners`](crate::job_runner::JobRunner).
pub struct ElectionTask {
    /// Etcd connections
    etcd_urls: Vec<String>,
    /// The etcd Client
    client: Client,
    /// The lease id of the current etcd lease
    lease: Lease,
    /// Sender to notify about election state changes
    state_sender: watch::Sender<ElectionState>,
    /// If received, the [`ElectionTask`] will become a follower in the cluster
    follow_cmd: mpsc::Receiver<()>,
}

impl ElectionTask {
    pub async fn start(etcd_urls: Vec<String>) -> Result<ElectionTaskHandle, ElectionTaskError> {
        let mut client = Client::connect(&etcd_urls, None)
            .await
            .context(ConnectSnafu)?;

        let lease = Lease::new(&mut client).await?;

        let (state_tx, state_rx) = watch::channel(ElectionState::Hold);
        let (cmd_tx, cmd_rx) = mpsc::channel(10);

        let task = Self {
            etcd_urls,
            client,
            lease,
            state_sender: state_tx,
            follow_cmd: cmd_rx,
        };

        tokio::spawn(async move {
            task.run().await;
        });

        Ok(ElectionTaskHandle {
            state: state_rx,
            follow_cmd: cmd_tx,
        })
    }

    async fn run(mut self) {
        loop {
            match self.run_inner().await {
                Ok(_) => {
                    // if the inner loop returns ok, some condition to gracefully shutdown has been met and this can exit
                    return;
                }
                Err(e) => {
                    log::error!("ElectionTask encountered error: {e:?}");
                    self.reconnect().await
                }
            }
        }
    }

    async fn run_inner(&mut self) -> Result<(), ElectionTaskError> {
        self.try_become_leader().await?;

        let mut watch_stream = self
            .client
            .watch(ELECTION_KEY, None)
            .await
            .context(CreateWatchSnafu)?;

        watch_stream
            .request_progress()
            .await
            .context(WatchProgressSnafu)?;

        let (mut lease_keeper, _) = self
            .client
            .lease_keep_alive(self.lease.id)
            .await
            .context(CreateKeepAliveSnafu)?;

        let mut keep_alive_interval = interval(Duration::from_secs(self.lease.ttl as u64 / 2));

        loop {
            tokio::select! {
                msg = watch_stream.message() => {
                    let msg = msg.context(WatchStreamSnafu)?.ok_or(EtcdError::WatchClosed)?;

                    self.handle_watch_msg(msg).await?
                }

                _ = keep_alive_interval.tick() => {
                    lease_keeper.keep_alive().await.context(KeepAliveSnafu)?;
                }

                follow_cmd = self.follow_cmd.recv() => {
                    if follow_cmd.is_none() {
                        // handle was dropped because the runner task exited, this is our signal to exit
                        log::info!("ElectionTask handle dropped, shutting down");
                        return Ok(())
                    }

                    self.step_down().await?;
                }
            }
        }
    }

    /// Set the leader key in etcd if it does not already exist
    ///
    /// If the key was set successfully, this ElectionTask becomes the new leader
    ///
    /// The leader key is automatically deleted by etcd when the lease of a ElectionTask expires
    async fn try_become_leader(&mut self) -> Result<(), ElectionTaskError> {
        let txn = etcd_client::Txn::new()
            .when([Compare::version(ELECTION_KEY, CompareOp::Equal, 0)])
            .and_then([TxnOp::put(
                ELECTION_KEY,
                self.lease.id.to_string(),
                Some(PutOptions::new().with_lease(self.lease.id)),
            )]);

        let response = self.client.txn(txn).await.context(AddSnafu {
            key: String::from_utf8_lossy(ELECTION_KEY),
        })?;

        if response.succeeded() {
            self.update_state(ElectionState::Leader);
        }

        Ok(())
    }

    async fn step_down(&mut self) -> Result<(), ElectionTaskError> {
        if *self.state_sender.borrow() != ElectionState::Leader {
            return Ok(());
        }

        self.state_sender.send_replace(ElectionState::Follower);

        self.client
            .delete(ELECTION_KEY, None)
            .await
            .context(RemoveSnafu {
                key: String::from_utf8_lossy(ELECTION_KEY),
            })?;

        Ok(())
    }

    /// Something about the leader key changed.
    ///
    /// When the leader key was deleted, we can attempt to be the first to set it again and thus become the new leader
    async fn handle_watch_msg(&mut self, msg: WatchResponse) -> Result<(), ElectionTaskError> {
        for event in msg.events() {
            if event.event_type() == EventType::Delete {
                self.try_become_leader().await?;
            }
        }

        Ok(())
    }

    fn update_state(&mut self, new: ElectionState) {
        self.state_sender.send_if_modified(|state| {
            if state != &new {
                *state = new;
                true
            } else {
                false
            }
        });
    }

    async fn reconnect(&mut self) {
        log::info!("ElectionTask entering reconnect state");
        self.update_state(ElectionState::Hold);

        loop {
            // wait for at least the amount of our lease ttl
            tokio::time::sleep(Duration::from_secs(self.lease.ttl as u64 + 2)).await;

            log::info!("Attempting to reconnect to etcd");

            let mut client = match Client::connect(&self.etcd_urls, None).await {
                Ok(client) => client,
                Err(err) => {
                    log::error!("Failed to create etcd client while reconnecting: {err}");
                    continue;
                }
            };

            let lease = match Lease::new(&mut client).await {
                Ok(lease) => lease,
                Err(err) => {
                    log::error!("Failed to get a new lease from etcd while reconnecting: {err}");
                    continue;
                }
            };

            log::info!("Successfully reconnected to etcd!");

            self.client = client;
            self.lease = lease;

            return;
        }
    }
}

/// A handle to interact with the ElectionTask
///
/// The ElectionTask exits when the handle is dropped
pub(crate) struct ElectionTaskHandle {
    /// Reflects the current [`ElectionState`] of the [`ElectionTask`]
    state: watch::Receiver<ElectionState>,
    /// Notify the election task to step down as leader and become follower
    follow_cmd: mpsc::Sender<()>,
}

impl ElectionTaskHandle {
    pub(crate) async fn become_follower(&self) -> Result<(), ElectionTaskError> {
        if *self.state.borrow() != ElectionState::Leader {
            return Ok(());
        }

        self.follow_cmd
            .send(())
            .await
            .context(ElectionTaskExitedSnafu)?;

        Ok(())
    }

    pub(crate) fn state_borrow(&self) -> Ref<'_, ElectionState> {
        self.state.borrow()
    }

    pub(crate) fn state_borrow_and_update(&mut self) -> Ref<'_, ElectionState> {
        self.state.borrow_and_update()
    }

    pub(crate) async fn state_changed(&mut self) -> Result<(), watch::error::RecvError> {
        self.state.changed().await
    }
}
