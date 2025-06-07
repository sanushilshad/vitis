// use std::{sync::Arc, time::Duration};

// use crate::websocket::{Server, WebSocketSession};
// use actix::Addr;
// use actix_web::web::{self, Data};
// use anyhow::anyhow;
// use futures::StreamExt;
// use rdkafka::{
//     ClientConfig, Message,
//     admin::{AdminClient, AdminOptions, NewTopic, TopicReplication},
//     consumer::{CommitMode, Consumer, StreamConsumer},
//     error::KafkaError,
//     producer::FutureProducer,
// };
// use reqwest::Client;
// use serde::{Deserialize, Serialize};
// use serde_json::Value;
// use sqlx::PgPool;

// use crate::{configuration::get_configuration, utils::pascal_to_snake_case};

// #[derive(Debug)]
// pub enum KafkaGroupName {
//     Notification,
// }

// impl std::fmt::Display for KafkaGroupName {
//     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//         write!(f, "{}", pascal_to_snake_case(&format!("{:?}", self)))
//     }
// }

// #[derive(Debug)]
// pub enum KafkaTopicName {
//     VitisNotification,
// }

// impl std::fmt::Display for KafkaTopicName {
//     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//         write!(f, "{}", pascal_to_snake_case(&format!("{:?}", self)))
//     }
// }

// pub struct KafkaClient {
//     servers: String,
//     pub environment: String,
//     pub producer: FutureProducer,
// }

// impl KafkaClient {
//     pub fn create_producer(servers: &str) -> FutureProducer {
//         ClientConfig::new()
//             .set("bootstrap.servers", servers)
//             .set("message.timeout.ms", "5000")
//             .create::<FutureProducer>()
//             .expect("Kafka Producer creation error")
//     }
//     pub fn new(servers: String, environment: String) -> Self {
//         let producer = Self::create_producer(&servers);
//         Self {
//             servers,
//             environment,
//             producer,
//         }
//     }
//     pub async fn create_topic(&self, topic_name: &str) -> Result<(), anyhow::Error> {
//         let admin_client: AdminClient<_> = ClientConfig::new()
//             .set("bootstrap.servers", &self.servers)
//             .create()
//             .expect("Failed to create Kafka AdminClient");

//         let new_topic = NewTopic::new(topic_name, 9, TopicReplication::Fixed(3));

//         let options = AdminOptions::new().operation_timeout(Some(Duration::from_secs(5)));
//         match admin_client.create_topics(&[new_topic], &options).await {
//             Ok(results) => {
//                 for result in results {
//                     match result {
//                         Ok(_) => println!("Topic '{}' created successfully", &topic_name),
//                         Err((topic_name, err)) => {
//                             return Err(anyhow!(
//                                 "Failed to create topic '{}': {:?}",
//                                 topic_name,
//                                 err
//                             ));
//                         }
//                     };
//                 }
//                 Ok(())
//             }
//             Err(err) => Err(anyhow!("Error during topic creation: {:?}", err)),
//         }
//     }
//     pub fn get_topic_name(&self, topic: KafkaTopicName) -> String {
//         format!("{}_{}", self.environment, topic)
//     }
//     pub async fn kafka_client_notification_consumer(
//         &self,
//         websocket_client: web::Data<Addr<Server>>,
//         pool: Data<PgPool>,
//     ) -> Result<(), KafkaError> {
//         let consumer: StreamConsumer = ClientConfig::new()
//             .set("bootstrap.servers", &self.servers)
//             .set("group.id", KafkaGroupName::Notification.to_string()) // Use WebSocket key as group.id
//             .set("enable.partition.eof", "false")
//             .set("session.timeout.ms", "6000")
//             .set("enable.auto.commit", "false")
//             .create()?;

//         consumer.subscribe(&[&self.get_topic_name(KafkaTopicName::VitisNotification)])?;

//         tokio::spawn(async move {
//             let mut message_stream = consumer.stream();

//             while let Some(result) = message_stream.next().await {
//                 match result {
//                     Ok(msg) => {
//                         if let Some(payload) = msg.payload() {
//                             if let Ok(message_data) =
//                                 serde_json::from_slice::<KafkaSearchData>(payload)
//                             {

//                                 {
//                                     Ok(_) => {
//                                         if let Err(e) =
//                                             consumer.commit_message(&msg, CommitMode::Async)
//                                         {
//                                             eprintln!("Failed to commit message: {:?}", e);
//                                         }
//                                     }
//                                     Err(e) => {
//                                         eprintln!("Error in process_on_search: {:?}", e);
//                                     }
//                                 }
//                             }
//                         }
//                     }
//                     Err(e) => {
//                         eprintln!("Error receiving message: {:?}", e);
//                     }
//                 }
//             }
//         });

//         Ok(())
//     }

//     pub async fn kafka_observability_consumer(
//         &self,
//         pool: Data<PgPool>,
//         map: Data<StartUpMap>,
//         ondc_obj: Data<ONDCConfig>,
//     ) -> Result<(), KafkaError> {
//         let consumer: StreamConsumer = ClientConfig::new()
//             .set("bootstrap.servers", &self.servers)
//             .set("group.id", KafkaGroupName::Observability.to_string()) // Use WebSocket key as group.id
//             .set("enable.partition.eof", "false")
//             .set("session.timeout.ms", "6000")
//             .set("enable.auto.commit", "false")
//             .create()?;

//         consumer.subscribe(&[&self.get_topic_name(KafkaTopicName::RetailB2BBuyerObservability)])?;
//         let client = Arc::new(Client::new());
//         let ondc_obj = ondc_obj.clone();
//         tokio::spawn(async move {
//             let mut message_stream = consumer.stream();

//             while let Some(result) = message_stream.next().await {
//                 // let ondc_obj_ref = ondc_obj.clone();
//                 match result {
//                     Ok(msg) => {
//                         if let Some(payload) = msg.payload() {
//                             if let Ok(message_data) =
//                                 serde_json::from_slice::<ObservabilityProducerData>(payload)
//                             {
//                                 let np_detail = get_np_detail(
//                                     &pool,
//                                     &map,
//                                     &message_data.subscriber_id,
//                                     &ONDCNetworkType::Bap,
//                                 )
//                                 .await;

//                                 if let Ok(Some(data)) = np_detail {
//                                     if let Some(observability_token) = data.observability_token {
//                                         for data in message_data.data {
//                                             tracing::info!("{}", data);
//                                             if let Err(e) = send_observability(
//                                                 &ondc_obj,
//                                                 &observability_token,
//                                                 data,
//                                                 &client,
//                                             )
//                                             .await
//                                             {
//                                                 eprintln!(
//                                                     "Failed to send observability data: {:?}",
//                                                     e
//                                                 );
//                                             }
//                                         }
//                                     }
//                                 } else if let Err(e) = np_detail {
//                                     eprintln!("Error receiving message: {:?}", e);
//                                 } else {
//                                     eprintln!("Invalid subscriber ID");
//                                 }
//                             }
//                         }
//                     }
//                     Err(e) => {
//                         eprintln!("Error receiving message: {:?}", e);
//                     }
//                 }
//             }
//         });

//         Ok(())
//     }
// }

// pub async fn create_kafka_topic_command() {
//     let configuration = get_configuration().expect("Failed to read configuration.");
//     let kafka_client = configuration.kafka.client();
//     let _ = kafka_client
//         .create_topic(&kafka_client.get_topic_name(KafkaTopicName::RetailB2BBuyerSearch))
//         .await;
//     let _ = kafka_client
//         .create_topic(&kafka_client.get_topic_name(KafkaTopicName::RetailB2BBuyerObservability))
//         .await;
// }
