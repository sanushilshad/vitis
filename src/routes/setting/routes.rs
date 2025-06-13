use actix_web::web;

use crate::{
    middlewares::{
        ProjectAccountValidation, ProjectPermissionValidation, UserPermissionValidation,
    },
    routes::project::schemas::PermissionType,
};

use super::handlers::{
    create_project_config_req, create_user_config_req, fetch_config_enums, fetch_global_setting,
    fetch_project_config_req, fetch_user_config_req, save_global_setting,
};

pub fn setting_routes(cfg: &mut web::ServiceConfig) {
    cfg.route(
        "/project/save",
        web::post()
            .to(create_project_config_req)
            .wrap(ProjectPermissionValidation {
                permission_list: vec![
                    PermissionType::CreateProjectSetting.to_string(),
                    PermissionType::CreateProjectSettingSelf.to_string(),
                ],
            })
            .wrap(ProjectAccountValidation),
    );
    cfg.route(
        "/user/save",
        web::post()
            .to(create_user_config_req)
            .wrap(UserPermissionValidation {
                permission_list: vec![
                    PermissionType::CreateUserSetting.to_string(),
                    PermissionType::CreateUserSettingSelf.to_string(),
                ],
            }),
    );
    cfg.route(
        "/project/fetch",
        web::post()
            .to(fetch_project_config_req)
            .wrap(ProjectAccountValidation),
    );
    cfg.route("/user/fetch", web::post().to(fetch_user_config_req));
    cfg.route(
        "/global/fetch",
        web::post()
            .to(fetch_global_setting)
            .wrap(UserPermissionValidation {
                permission_list: vec![PermissionType::CreateGlobalSetting.to_string()],
            }),
    );
    cfg.route(
        "/global/save",
        web::post()
            .to(save_global_setting)
            .wrap(UserPermissionValidation {
                permission_list: vec![PermissionType::CreateGlobalSetting.to_string()],
            }),
    );
    cfg.route("/enum/fetch", web::post().to(fetch_config_enums));
}
