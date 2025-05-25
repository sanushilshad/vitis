use actix_web::web;

use crate::{middlewares::ProjectPermissionValidation, routes::project::schemas::PermissionType};

use super::handlers::{create_config_req, fetch_config_req};

pub fn setting_routes(cfg: &mut web::ServiceConfig) {
    cfg.route(
        "/create",
        web::post()
            .to(create_config_req)
            .wrap(ProjectPermissionValidation {
                permission_list: vec![PermissionType::CreateSetting.to_string()],
            }),
    );
    cfg.route("/fetch", web::post().to(fetch_config_req));
}
