use crate::{
    middlewares::{RequireAuth, UserPermissionValidation},
    schemas::PermissionType,
};

use actix_web::web;
pub fn role_routes(cfg: &mut web::ServiceConfig) {
    // cfg.route("/register", web::post().to(register_user_account_req));
}
