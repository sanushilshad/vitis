// use std::time::{Duration, SystemTime};

use crate::errors::GenericError;
use crate::pulsar_client::{PulsarClient, PulsarTopic, WSMessageData};
// use crate::routes::leave::utils::send_slack_notification_for_approved_leave;
use crate::schemas::{WSKeyTrait, WebSocketParam};
// use crate::slack_client::SlackClient;
use crate::websocket_client::{Server, WebSocketSession};
use actix::Addr;
// use actix_http::StatusCode;
use actix_web::{Error, HttpRequest, HttpResponse, web};
use actix_web_actors::ws;
// use chrono::{DateTime, Utc};
// use sqlx::PgPool;

#[utoipa::path(
    get,
    path = "/websocket",
    tag = "WebSocket",
    description = "For Order flow the WebSocket should only send the business_id, for Product search all the three paramters are required.",
    summary = "Connect WebSocket API",
    params(
        ("device_id" = Option<String>, Query, description = "Device Id"),
        ("user_id" = Option<String>, Query, description = "User Id"),
        ("business_id" = String, Query, description = "Business Id"),
    )
)]
#[tracing::instrument(
    name = "Commence web socket",
    skip(stream, server_addr),
    fields(producer_client)
)]
pub async fn web_socket(
    req: HttpRequest,
    stream: web::Payload,
    query: web::Query<WebSocketParam>,
    server_addr: web::Data<Addr<Server>>,
    producer_client: web::Data<PulsarClient>,
) -> Result<HttpResponse, Error> {
    let web_socket_key = query.get_ws_key();
    let res = ws::start(
        WebSocketSession::new(web_socket_key.to_string(), server_addr.get_ref().clone()),
        &req,
        stream,
    )?;

    let mut producer = producer_client
        .get_producer(producer_client.get_product_topic(PulsarTopic::WebSocket))
        .await;
    producer
        .send_non_blocking(WSMessageData {
            partition_key: web_socket_key.to_string(),
        })
        .await
        .map_err(|e| GenericError::UnexpectedError(e.into()))?;
    Ok(res)
}

// pub async fn slack_notification(
//     pool: web::Data<PgPool>,
//     slack_client: web::Data<SlackClient>,
//     producer_client: web::Data<PulsarClient>,
// ) -> Result<HttpResponse, Error> {
//     let mut producer = producer_client
//         .get_producer(producer_client.get_topic_name(PulsarTopic::Scheduler))
//         .await;

//     // let message = "APPLE".to_string();

//     // // Set delivery time to now + 3 minutes
//     let deliver_at = SystemTime::now() + Duration::from_secs(10);
//     // let deliver_at_1 = Utc::now() + Duration::from_secs(100);
//     let date_string = "2025-07-03 00:00:00+00:00";
//     let deliver_at_1 = date_string
//         .parse::<DateTime<Utc>>()
//         .expect("Invalid datetime format");
//     let system_time: SystemTime = deliver_at_1.into();
//     let msg = SchedulerMessageData {
//         partition_key: None,
//         date: deliver_at_1,
//     };
//     let msg = producer
//         .create_message()
//         .with_content(msg)
//         .deliver_at(deliver_at)
//         .map_err(|e| GenericError::UnexpectedError(e.into()))?;
//     msg.send_non_blocking()
//         .await
//         .map_err(|e| GenericError::UnexpectedError(e.into()))?;
//     // let date_string = "2025-07-03 00:00:00+00:00";
//     // let datetime: DateTime<Utc> = date_string
//     //     .parse::<DateTime<Utc>>()
//     //     .expect("Invalid datetime format");

//     // if let Err(err) =
//     //     send_slack_notification_for_approved_leave(&pool, &slack_client, datetime).await
//     // {
//     //     return Ok(HttpResponse::new(StatusCode::INTERNAL_SERVER_ERROR));
//     // }

//     Ok(HttpResponse::new(StatusCode::ACCEPTED))
// }
