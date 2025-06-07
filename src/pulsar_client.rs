use crate::{
    utils::{delete_notifications_by_connection_id, fetch_notifications_by_connection_id},
    websocket::{Server, SessionExists},
};
use actix::Addr;
use actix_web::web;
use actix_web::web::Data;
use anyhow::Context;
use futures::TryStreamExt;
use pulsar::{
    Consumer, DeserializeMessage, Error as PulsarError, Payload, Producer, Pulsar,
    SerializeMessage, SubType, TokioExecutor, producer,
};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use std::time::Duration;
use tokio::sync::Mutex;
#[derive(Debug, Deserialize, Serialize)]
pub struct MessageData {
    pub partition_key: String,
}

impl SerializeMessage for MessageData {
    fn serialize_message(input: Self) -> Result<producer::Message, PulsarError> {
        let payload = serde_json::to_vec(&input).map_err(|e| PulsarError::Custom(e.to_string()))?;
        Ok(producer::Message {
            payload,
            partition_key: Some(input.partition_key),
            ..Default::default()
        })
    }
}

impl DeserializeMessage for MessageData {
    type Output = Result<MessageData, serde_json::Error>;

    fn deserialize_message(payload: &Payload) -> Self::Output {
        serde_json::from_slice(&payload.data)
    }
}

pub struct AppState {
    pub producer: Mutex<Producer<TokioExecutor>>,
}

pub struct PulsarClient {
    client: Pulsar<TokioExecutor>,
    topic_name: String,
}

impl PulsarClient {
    #[tracing::instrument]
    pub async fn new(url: String, topic_name: String) -> Result<Self, pulsar::Error> {
        tracing::info!("Establishing connection to the Pulsar server.");
        let client = Pulsar::builder(url, TokioExecutor).build().await?;
        Ok(Self { client, topic_name })
    }

    pub fn get_partiton_topic(&self, ws_key_id: &str) -> String {
        format!("{}-partition-{}", &self.topic_name, ws_key_id)
    }

    fn get_product_topic(&self) -> String {
        format!("persistent://public/default/{}", &self.topic_name)
    }

    pub async fn get_producer(&self) -> Producer<TokioExecutor> {
        self.client
            .producer()
            .with_topic(self.get_product_topic())
            .build()
            .await
            .expect("Failed to create producer")
    }

    pub async fn get_consumer(
        &self,
        consumer_name: String,
        subscription: String,
    ) -> Consumer<MessageData, TokioExecutor> {
        let consumer: Consumer<MessageData, TokioExecutor> = self
            .client
            .consumer()
            .with_topic(self.topic_name.clone())
            .with_consumer_name(consumer_name)
            .with_subscription_type(SubType::KeyShared)
            .with_subscription(subscription)
            .with_unacked_message_resend_delay(Some(Duration::from_secs(10)))
            .build()
            .await
            .expect("Failed to create consumer");
        consumer
    }

    pub async fn start_consumer(
        &self,
        pool: web::Data<PgPool>,

        mut consumer: Consumer<MessageData, TokioExecutor>,

        websocket_client: Data<Addr<Server>>,
    ) {
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
                                if let Ok(a) = delete_notifications_by_connection_id(
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
}
