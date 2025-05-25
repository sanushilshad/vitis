use super::handlers::{authenticate_req, fetch_user_req, register_user_account_req, send_otp_req};
use crate::middlewares::RequireAuth;

use actix_web::web;
pub fn user_routes(cfg: &mut web::ServiceConfig) {
    cfg.route("/register", web::post().to(register_user_account_req))
        .route("/fetch", web::post().to(fetch_user_req).wrap(RequireAuth))
        .route("/authenticate", web::post().to(authenticate_req))
        .route("/otp/send", web::post().to(send_otp_req));
}
