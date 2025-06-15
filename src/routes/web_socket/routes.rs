use actix_web::web;

use super::handlers::web_socket;

pub fn web_socket_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(web::resource("").route(web::get().to(web_socket)));
    // cfg.service(web::resource("/slack").route(web::get().to(slack_notification)));
}
