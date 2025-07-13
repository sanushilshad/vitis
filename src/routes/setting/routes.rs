use actix_web::web;

use crate::{
    middlewares::{
        BusinessAccountValidation, BusinessPermissionValidation, UserPermissionValidation,
    },
    schemas::PermissionType,
};

use super::handlers::{
    create_business_config_req, create_user_business_config_req, create_user_config_req,
    list_business_config_req, list_config_enums, list_global_setting,
    list_user_business_config_req, list_user_config_req, save_global_setting,
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
        "/business/list",
        web::post()
            .to(list_business_config_req)
            .wrap(BusinessPermissionValidation {
                permission_list: vec![
                    PermissionType::CreateBusinessSetting.to_string(),
                    PermissionType::CreateBusinessSettingSelf.to_string(),
                ],
            })
            .wrap(BusinessAccountValidation),
    );
    cfg.route(
        "/user/list",
        web::post()
            .to(list_user_config_req)
            .wrap(UserPermissionValidation {
                permission_list: vec![
                    PermissionType::CreateUserSetting.to_string(),
                    PermissionType::CreateUserSettingSelf.to_string(),
                ],
            }),
    );
    cfg.route(
        "/global/list",
        web::post()
            .to(list_global_setting)
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
    cfg.route(
        "/user-business/list",
        web::post()
            .to(list_user_business_config_req)
            .wrap(BusinessPermissionValidation {
                permission_list: vec![PermissionType::CreateUserBusinessSetting.to_string()],
            })
            .wrap(BusinessAccountValidation),
    );
    cfg.route(
        "/user-business/save",
        web::post()
            .to(create_user_business_config_req)
            .wrap(BusinessPermissionValidation {
                permission_list: vec![PermissionType::CreateUserBusinessSetting.to_string()],
            })
            .wrap(BusinessAccountValidation),
    );
    cfg.route("/enum/list", web::post().to(list_config_enums));
}
