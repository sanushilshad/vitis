use crate::{
    routes::leave::utils::send_slack_notification_for_approved_leave,
    slack_client::SlackClient,
    utils::{delete_notifications_by_connection_id, fetch_notifications_by_connection_id},
    websocket_client::{Server, SessionExists},
};
use actix::Addr;
use actix_web::web;
use actix_web::web::Data;
use anyhow::Context;
use chrono::{DateTime, Utc};
use futures::TryStreamExt;
use pulsar::{
    Consumer, DeserializeMessage, Error as PulsarError, Payload, Producer, Pulsar,
    SerializeMessage, SubType, TokioExecutor, producer,
};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use std::{fmt, time::Duration};

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "snake_case")]
pub enum PulsarTopic {
    WebSocket,
    Scheduler,
}

impl fmt::Display for PulsarTopic {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let display_str = match self {
            PulsarTopic::WebSocket => "web_socket",
            PulsarTopic::Scheduler => "scheduler",
        };
        write!(f, "{}", display_str)
    }
}

// use tokio::sync::Mutex;
#[derive(Debug, Deserialize, Serialize)]
pub struct WSMessageData {
    pub partition_key: String,
}

impl SerializeMessage for WSMessageData {
    fn serialize_message(input: Self) -> Result<producer::Message, PulsarError> {
        let payload = serde_json::to_vec(&input).map_err(|e| PulsarError::Custom(e.to_string()))?;
        Ok(producer::Message {
            payload,
            partition_key: Some(input.partition_key),
            ..Default::default()
        })
    }
}

impl DeserializeMessage for WSMessageData {
    type Output = Result<WSMessageData, serde_json::Error>;

    fn deserialize_message(payload: &Payload) -> Self::Output {
        serde_json::from_slice(&payload.data)
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct SchedulerMessageData {
    pub partition_key: Option<String>,
    pub date: DateTime<Utc>,
}

impl SerializeMessage for SchedulerMessageData {
    fn serialize_message(input: Self) -> Result<producer::Message, PulsarError> {
        let payload = serde_json::to_vec(&input).map_err(|e| PulsarError::Custom(e.to_string()))?;
        Ok(producer::Message {
            payload,
            partition_key: input.partition_key,
            ..Default::default()
        })
    }
}

impl DeserializeMessage for SchedulerMessageData {
    type Output = Result<SchedulerMessageData, serde_json::Error>;

    fn deserialize_message(payload: &Payload) -> Self::Output {
        serde_json::from_slice(&payload.data)
    }
}

pub struct AppState {
    pub producer: Producer<TokioExecutor>,
}

pub struct PulsarClient {
    client: Pulsar<TokioExecutor>,
    topic_prefix: String,
}

impl PulsarClient {
    #[tracing::instrument]
    pub async fn new(url: String, topic_prefix: String) -> Result<Self, pulsar::Error> {
        tracing::info!("Establishing connection to the Pulsar server.");
        let client = Pulsar::builder(url, TokioExecutor).build().await?;
        Ok(Self {
            client,
            topic_prefix,
        })
    }

    pub fn get_topic_name(&self, topic: PulsarTopic) -> String {
        format!("{}_{}", &self.topic_prefix, topic)
    }
    pub fn get_product_topic(&self, topic_name: PulsarTopic) -> String {
        let topic_name = self.get_topic_name(topic_name);
        format!("persistent://public/default/{}", topic_name)
    }
    // pub fn get_partiton_topic(&self, topic_name: &str, ws_key_id: &str) -> String {
    //     format!("{}-partition-{}", topic_name, ws_key_id)
    // }

    pub async fn get_producer(&self, topic_name: String) -> Producer<TokioExecutor> {
        self.client
            .producer()
            .with_topic(topic_name)
            .build()
            .await
            .expect("Failed to create producer")
    }

    pub async fn initiate_cosumer<T>(
        &self,
        consumer_name: &str,
        subscription: &str,
        topic_name: &str,
    ) -> Consumer<T, TokioExecutor>
    where
        T: DeserializeMessage,
    {
        let consumer: Consumer<T, TokioExecutor> = self
            .client
            .consumer()
            .with_topic(topic_name)
            .with_consumer_name(consumer_name)
            .with_subscription_type(SubType::KeyShared)
            .with_subscription(subscription)
            .with_unacked_message_resend_delay(Some(Duration::from_secs(10)))
            .build()
            .await
            .expect("Failed to create consumer");
        consumer
    }

    pub async fn start_ws_consumer(
        &self,
        consumer_name: &str,
        subscription: &str,
        pool: web::Data<PgPool>,
        topic_name: &str,

        websocket_client: Data<Addr<Server>>,
    ) {
        let mut consumer = self
            .initiate_cosumer::<WSMessageData>(consumer_name, subscription, topic_name)
            .await;
        tokio::spawn(async move {
            while let Some(result) = consumer.try_next().await.transpose() {
                match result {
                    Ok(msg) => {
                        let partition_key = msg.metadata().partition_key();
                        if websocket_client
                            .send(SessionExists {
                                id: partition_key.to_owned(),
                            })
                            .await
                            .unwrap_or(false)
                        {
                            let mut transaction = pool
                                .begin()
                                .await
                                .context("Failed to acquire a Postgres connection from the pool")
                                .unwrap();
                            if let Ok(notifications) = fetch_notifications_by_connection_id(
                                &mut transaction,
                                partition_key,
                            )
                            .await
                            {
                                for notification in notifications.iter() {
                                    websocket_client.do_send(notification.data.0.clone());
                                }
                                if let Err(a) = delete_notifications_by_connection_id(
                                    &mut transaction,
                                    partition_key,
                                )
                                .await
                                {
                                    eprintln!("Failed to deleted message: {:?}", a);
                                }
                            }
                            transaction
                                .commit()
                                .await
                                .context("Failed to commit SQL transaction to store a order")
                                .unwrap();

                            if let Err(e) = consumer.ack(&msg).await {
                                eprintln!("Failed to acknowledge message: {:?}", e);
                            }
                        }
                    }
                    Err(e) => {
                        eprintln!("Failed to receive message: {:?}", e);
                    }
                }
            }
        });
    }

    pub async fn start_scheduler_consumer(
        &self,
        consumer_name: &str,
        subscription: &str,
        pool: web::Data<PgPool>,
        topic_name: &str,
        slack_client: Data<SlackClient>,
    ) {
        let mut consumer = self
            .initiate_cosumer::<SchedulerMessageData>(consumer_name, subscription, topic_name)
            .await;

        tokio::spawn(async move {
            while let Some(result) = consumer.try_next().await.transpose() {
                match result {
                    Ok(msg) => match msg.deserialize() {
                        Ok(data) => {
                            if let Err(e) = send_slack_notification_for_approved_leave(
                                &pool,
                                &slack_client,
                                data.date,
                            )
                            .await
                            {
                                eprintln!("Failed to send Slack message: {:?}", e);
                            } else if let Err(e) = consumer.ack(&msg).await {
                                eprintln!("Failed to acknowledge message: {:?}", e);
                            }
                        }
                        Err(e) => {
                            eprintln!("Failed to deserialize message payload: {:?}", e);
                        }
                    },
                    Err(e) => {
                        eprintln!("Failed to receive message: {:?}", e);
                    }
                }
            }
        });
    }
}
