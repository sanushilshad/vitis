use actix_web::web;

use crate::{
    middlewares::{
        BusinessAccountValidation, BusinessPermissionValidation, UserPermissionValidation,
    },
    schemas::PermissionType,
};

use super::handlers::{
    create_business_config_req, create_user_config_req, fetch_business_config_req,
    fetch_config_enums, fetch_global_setting, fetch_user_config_req, save_global_setting,
};

pub fn setting_routes(cfg: &mut web::ServiceConfig) {
    cfg.route(
        "/business/save",
        web::post()
            .to(create_business_config_req)
            .wrap(BusinessPermissionValidation {
                permission_list: vec![
                    PermissionType::CreateBusinessSetting.to_string(),
                    PermissionType::CreateBusinessSettingSelf.to_string(),
                ],
            })
            .wrap(BusinessAccountValidation),
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
        "/business/fetch",
        web::post()
            .to(fetch_business_config_req)
            .wrap(BusinessAccountValidation),
    );
    cfg.route(
        "/user/fetch",
        web::post()
            .to(fetch_user_config_req)
            .wrap(UserPermissionValidation {
                permission_list: vec![
                    PermissionType::CreateUserSetting.to_string(),
                    PermissionType::CreateUserSettingSelf.to_string(),
                ],
            }),
    );
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
