use actix_web::web;

use super::handlers::{
    fetch_project_req, list_project_req, project_permission_validation,
    register_project_account_req,
};

pub fn project_routes(cfg: &mut web::ServiceConfig) {
    cfg.route("/register", web::post().to(register_project_account_req))
        .route("/fetch", web::post().to(fetch_project_req))
        .route("/list", web::get().to(list_project_req))
        .route("/permission", web::post().to(project_permission_validation));
}
