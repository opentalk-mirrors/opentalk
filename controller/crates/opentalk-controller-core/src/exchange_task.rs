// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

//! Controller to controller messaging

use core::{hash::BuildHasherDefault, time::Duration};
use std::{borrow::Cow, collections::HashMap, num::NonZero, sync::Arc};

use bytestring::ByteString;
use lapin::{
    BasicProperties, ChannelState, Consumer, ExchangeKind,
    message::Delivery,
    options::{
        BasicConsumeOptions, BasicPublishOptions, ExchangeDeclareOptions, QueueBindOptions,
        QueueDeclareOptions,
    },
    types::FieldTable,
};
use lapin_pool::{RabbitMqChannel, RabbitMqPool};
use rustc_hash::FxHasher;
use serde::{Deserialize, Serialize};
use slotmap::{SlotMap, new_key_type};
use snafu::{Report, Snafu};
use tokio::{
    select,
    sync::{mpsc, oneshot},
    time::sleep,
};
use tokio_stream::StreamExt;
use uuid::Uuid;

#[derive(Debug, Snafu)]
pub enum Error {
    #[snafu(display("failed open channel"), context(false))]
    CreateChannel { source: lapin_pool::Error },

    #[snafu(display("failed to make consumer"), context(false))]
    Consumer { source: lapin::Error },

    #[snafu(display("received invalid message from rabbitMQ"), context(false))]
    InvalidRmqMessage { source: serde_json::Error },
}

const EXCHANGE: &str = "opentalk_controller";

new_key_type! {
    struct SubscriberKey;
}

/// Task that runs in the background. Receives messages from RabbitMQ and distributes them to its subscribers.
///
/// This exchange task works similar to the RabbitMQ topic exchange. Subscribers register routing-keys (topics) that
/// they are interested in and want to receive messages from.
///
/// Messages sent using the [`ExchangeHandle`] will be distributed internally before being sent to other controllers
/// via a RabbitMQ fanout exchange.
pub struct ExchangeTask {
    /// Some unique ID to identify messages sent by ourselves
    id: Uuid,

    rmq: Option<RabbitMQParts>,

    command_receiver: mpsc::UnboundedReceiver<Command>,

    /// List of subscribers
    subscriber: SlotMap<SubscriberKey, SubscriberEntry>,

    /// Mapping of routing-key -> Subscribers
    ///
    /// This map is used to match the routing key of incoming messages to subscribers.
    routing_keys: HashMap<ByteString, Vec<SubscriberKey>, BuildHasherDefault<FxHasher>>,
}

struct RabbitMQParts {
    pool: Arc<RabbitMqPool>,
    channel: RabbitMqChannel,
    consumer: Consumer,
}

enum Command {
    CreateSubscriber {
        ret: oneshot::Sender<(SubscriberKey, mpsc::Receiver<ByteString>)>,
        routing_keys: Vec<ByteString>,
    },
    DropSubscriber(SubscriberKey),
    Publish {
        routing_key: String,
        ttl_milliseconds: Option<NonZero<u64>>,
        data: String,
    },
}

struct SubscriberEntry {
    sender: mpsc::Sender<ByteString>,
    routing_keys: Vec<ByteString>,
}

impl ExchangeTask {
    /// Spawn the exchange task and return a [`ExchangeHandle`] to it
    pub async fn spawn() -> Result<ExchangeHandle, Error> {
        let (command_sender, command_receiver) = mpsc::unbounded_channel();

        tokio::spawn(
            ExchangeTask {
                id: Uuid::new_v4(),
                rmq: None,
                command_receiver,
                subscriber: SlotMap::with_key(),
                routing_keys: HashMap::default(),
            }
            .run(),
        );

        Ok(ExchangeHandle { command_sender })
    }

    /// Spawn the exchange task with rabbitmq support and return a [`ExchangeHandle`] to it
    pub async fn spawn_with_rabbitmq(pool: Arc<RabbitMqPool>) -> Result<ExchangeHandle, Error> {
        let channel = pool.create_channel().await?;
        let consumer = make_consumer(&channel).await?;

        let (command_sender, command_receiver) = mpsc::unbounded_channel();

        tokio::spawn(
            ExchangeTask {
                id: Uuid::new_v4(),
                rmq: Some(RabbitMQParts {
                    pool,
                    channel,
                    consumer,
                }),
                command_receiver,
                subscriber: SlotMap::with_key(),
                routing_keys: HashMap::default(),
            }
            .run(),
        );

        Ok(ExchangeHandle { command_sender })
    }

    /// Loop forever and handle event
    async fn run(mut self) {
        loop {
            if let Some(rmq) = &mut self.rmq {
                select! {
                    command = self.command_receiver.recv() => {
                        // Return if command is None, all handles are dropped
                        let Some(command) = command else { return; };

                        self.handle_command(command).await;
                    }
                    delivery = rmq.consumer.next() => {
                        self.handle_delivery(delivery).await;
                    }
                }
            } else if let Some(command) = self.command_receiver.recv().await {
                self.handle_command(command).await;
            } else {
                // All handles dropped, exit
                return;
            }
        }
    }

    /// Handle an incoming command from a [`ExchangeHandle`]
    async fn handle_command(&mut self, command: Command) {
        match command {
            Command::CreateSubscriber { ret, routing_keys } => {
                let (sender, receiver) = mpsc::channel(2);
                let key = self.subscriber.insert(SubscriberEntry {
                    sender,
                    routing_keys: routing_keys.clone(),
                });

                if ret.send((key, receiver)).is_err() {
                    self.subscriber.remove(key);
                } else {
                    for routing_key in routing_keys {
                        self.routing_keys.entry(routing_key).or_default().push(key);
                    }
                }
            }
            Command::DropSubscriber(key) => {
                if let Some(entry) = self.subscriber.remove(key) {
                    for routing_key in entry.routing_keys {
                        if let Some(subscriber_keys) = self.routing_keys.get_mut(&routing_key) {
                            subscriber_keys.retain(|&k| k != key);

                            if subscriber_keys.is_empty() {
                                self.routing_keys.remove(&routing_key);
                            }
                        }
                    }
                }
            }
            Command::Publish {
                routing_key,
                ttl_milliseconds,
                data,
            } => {
                self.handle_msg(&routing_key, &data).await;

                let Some(mut rmq) = self.rmq.as_mut() else {
                    return;
                };

                let payload = serde_json::to_string(&RabbitMqMessage {
                    sender: self.id,
                    routing_key: &routing_key,
                    data: Cow::Owned(data),
                })
                .unwrap();

                let properties = if let Some(ttl_milliseconds) = ttl_milliseconds {
                    BasicProperties::default().with_expiration(ttl_milliseconds.to_string().into())
                } else {
                    BasicProperties::default()
                };

                // Reconnect while this fails with an error
                while rmq
                    .channel
                    .basic_publish(
                        EXCHANGE,
                        "",
                        BasicPublishOptions::default(),
                        payload.as_bytes(),
                        properties.clone(),
                    )
                    .await
                    .is_err()
                    && rmq.channel.status().state() == ChannelState::Error
                {
                    self.reconnect_rabbitmq().await;
                    rmq = self.rmq.as_mut().unwrap();
                }
            }
        }
    }

    async fn handle_delivery(&mut self, delivery: Option<lapin::Result<Delivery>>) {
        let Some(rmq) = &mut self.rmq else { return };

        match delivery {
            Some(Ok(delivery)) => {
                if let Err(e) = self.handle_rmq_message(&delivery.data).await {
                    tracing::warn!(
                        "Failed to handle incoming RMQ message, {}",
                        Report::from_error(e)
                    );
                }
            }
            Some(Err(_)) | None => {
                if rmq.channel.status().state() == ChannelState::Error {
                    self.reconnect_rabbitmq().await;
                }
            }
        }
    }

    async fn reconnect_rabbitmq(&mut self) {
        tracing::error!("Disconnected from RabbitMQ! Trying to reconnect");

        // Reset all subscribers this will cause all handles to receive an error
        self.subscriber.clear();
        self.routing_keys.clear();

        let mut wait_duration = Duration::from_millis(100);

        let Some(rmq) = &mut self.rmq else { return };

        loop {
            sleep(wait_duration).await;

            match rmq.pool.create_channel().await {
                Ok(channel) => match make_consumer(&channel).await {
                    Ok(consumer) => {
                        rmq.channel = channel;
                        rmq.consumer = consumer;
                        tracing::info!("Reconnected to RabbitMQ");
                        return;
                    }
                    Err(e) => tracing::warn!(
                        "Was able to create channel but not consumer, {}",
                        Report::from_error(e)
                    ),
                },
                Err(_) => {
                    tracing::warn!(
                        "RabbitMQ reconnect attempt failed, waiting {wait_duration:?} before next attempt"
                    );
                }
            }

            wait_duration = (wait_duration * 2).min(Duration::from_secs(2))
        }
    }

    async fn handle_rmq_message(&mut self, data: &[u8]) -> Result<(), Error> {
        let wrapper: RabbitMqMessage = serde_json::from_slice(data)?;

        if wrapper.sender == self.id {
            // Do not handle messages sent by us
            return Ok(());
        }

        self.handle_msg(wrapper.routing_key, &wrapper.data).await;

        Ok(())
    }

    async fn handle_msg(&mut self, routing_key: &str, data: &str) {
        if let Some(subscriber_keys) = self.routing_keys.get(routing_key) {
            let data = ByteString::from(data);

            for key in subscriber_keys {
                let _ = self.subscriber[*key].sender.send(data.clone()).await;
            }
        }
    }
}

async fn make_consumer(channel: &lapin::Channel) -> Result<Consumer, Error> {
    channel
        .exchange_declare(
            EXCHANGE,
            ExchangeKind::Fanout,
            ExchangeDeclareOptions::default(),
            FieldTable::default(),
        )
        .await?;

    let queue = channel
        .queue_declare(
            "",
            QueueDeclareOptions {
                exclusive: true,
                auto_delete: true,
                ..Default::default()
            },
            FieldTable::default(),
        )
        .await?;

    channel
        .queue_bind(
            queue.name().as_str(),
            EXCHANGE,
            "",
            QueueBindOptions::default(),
            FieldTable::default(),
        )
        .await?;

    let consumer = channel
        .basic_consume(
            queue.name().as_str(),
            "",
            BasicConsumeOptions {
                no_ack: true,
                exclusive: true,
                ..Default::default()
            },
            FieldTable::default(),
        )
        .await?;

    Ok(consumer)
}

/// Handle to the [`ExchangeTask`], used to create subscriber and publish messages
#[derive(Clone)]
pub struct ExchangeHandle {
    command_sender: mpsc::UnboundedSender<Command>,
}

#[derive(Debug, Snafu)]
#[snafu(display("Cannot create subscriber. Exchange task is not available, receiver dropped"))]
pub struct CreateSubscriberError;

#[derive(Debug, Snafu)]
#[snafu(display("Cannot publish message. Exchange task is not available, receiver dropped"))]
pub struct PublishError;

impl ExchangeHandle {
    /// Create a new [`SubscriberHandle`] which will receive all messages that match the given `routing_keys`
    ///
    /// Returns a error when the exchange is not reachable. The error is permanent.
    pub async fn create_subscriber(
        &self,
        routing_keys: Vec<String>,
    ) -> Result<SubscriberHandle, CreateSubscriberError> {
        let (ret, recv) = oneshot::channel();

        self.command_sender
            .send(Command::CreateSubscriber {
                ret,
                routing_keys: routing_keys.into_iter().map(ByteString::from).collect(),
            })
            .map_err(|_| CreateSubscriberError)?;

        let (key, receiver) = recv.await.map_err(|_| CreateSubscriberError)?;

        Ok(SubscriberHandle {
            exchange_handle: self.clone(),
            key,
            receiver,
        })
    }

    /// Publish a message to the exchange
    ///
    /// Returns an error when the exchange is not reachable. The error is permanent.
    pub fn publish(
        &self,
        routing_key: impl Into<String>,
        ttl_milliseconds: Option<NonZero<u64>>,
        data: impl Into<String>,
    ) -> Result<(), PublishError> {
        self.command_sender
            .send(Command::Publish {
                routing_key: routing_key.into(),
                ttl_milliseconds,
                data: data.into(),
            })
            .map_err(|_| PublishError)
    }

    #[cfg(feature = "mocking")]
    pub fn dummy() -> Self {
        let (command_sender, _) = mpsc::unbounded_channel();
        Self { command_sender }
    }
}

/// Handle to a subscriber on the exchange task
pub struct SubscriberHandle {
    exchange_handle: ExchangeHandle,
    key: SubscriberKey,
    receiver: mpsc::Receiver<ByteString>,
}

impl SubscriberHandle {
    /// Receive the next message
    ///
    /// Return None if the subscriber was removed and therefore no longer valid.
    pub async fn receive(&mut self) -> Option<ByteString> {
        self.receiver.recv().await
    }
}

impl Drop for SubscriberHandle {
    fn drop(&mut self) {
        let _ = self
            .exchange_handle
            .command_sender
            .send(Command::DropSubscriber(self.key));
    }
}

/// Format of RabbitMQ messages sent through the fanout exchange to other controllers
#[derive(Serialize, Deserialize)]
struct RabbitMqMessage<'w> {
    sender: Uuid,
    routing_key: &'w str,
    data: Cow<'w, str>,
}
