use actix_web::web;

use crate::{
    middlewares::{ProjectAccountValidation, ProjectPermissionValidation},
    routes::project::schemas::PermissionType,
};

use super::handlers::{
    create_project_config_req, create_user_config_req, fetch_project_config_req,
    fetch_user_config_req,
};

pub fn setting_routes(cfg: &mut web::ServiceConfig) {
    cfg.route(
        "/project/create",
        web::post()
            .to(create_project_config_req)
            .wrap(ProjectPermissionValidation {
                permission_list: vec![PermissionType::CreateSetting.to_string()],
            })
            .wrap(ProjectAccountValidation),
    );
    cfg.route("/user/create", web::post().to(create_user_config_req));
    cfg.route(
        "/project/fetch",
        web::post()
            .to(fetch_project_config_req)
            .wrap(ProjectAccountValidation),
    );
    cfg.route("/user/fetch", web::post().to(fetch_user_config_req));
}
