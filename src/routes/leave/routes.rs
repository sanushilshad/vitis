use actix_web::web;

use super::handlers::create_leave_req;

pub fn leave_routes(cfg: &mut web::ServiceConfig) {
    cfg.route("/create", web::post().to(create_leave_req));
    cfg.route("/delete", web::delete().to(create_leave_req));
}
