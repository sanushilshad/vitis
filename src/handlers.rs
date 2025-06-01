use crate::schemas::{WSKeyTrait, WebSocketParam};
use crate::websocket::{Server, WebSocketSession};
use actix::Addr;
use actix_web::{Error, HttpRequest, HttpResponse, Responder, web};
use actix_web_actors::ws;

pub async fn health_check() -> impl Responder {
    HttpResponse::Ok().body("Running Server")
}

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
) -> Result<HttpResponse, Error> {
    let web_socket_key = query.get_ws_key();
    let res = ws::start(
        WebSocketSession::new(web_socket_key.to_string(), server_addr.get_ref().clone()),
        &req,
        stream,
    )?;
    Ok(res)
}
