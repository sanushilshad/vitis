use crate::{
    middlewares::{BusinessAccountValidation, BusinessPermissionValidation},
    schemas::PermissionType,
};

use actix_web::web;

use super::handlers::{
    associate_permissions_to_role, disassociate_permissions_to_role, list_business_permissions,
};
pub fn permission_routes(cfg: &mut web::ServiceConfig) {
    cfg.route(
        "/business/list",
        web::post()
            .to(list_business_permissions)
            .wrap(BusinessPermissionValidation {
                permission_list: vec![PermissionType::CreateBusinessRole.to_string()],
            })
            .wrap(BusinessAccountValidation),
    );
    cfg.route(
        "/business-role/associate",
        web::post()
            .to(associate_permissions_to_role)
            .wrap(BusinessPermissionValidation {
                permission_list: vec![PermissionType::CreateBusinessRole.to_string()],
            })
            .wrap(BusinessAccountValidation),
    );
    cfg.route(
        "/business-role/disassociate",
        web::delete()
            .to(disassociate_permissions_to_role)
            .wrap(BusinessPermissionValidation {
                permission_list: vec![PermissionType::CreateBusinessRole.to_string()],
            })
            .wrap(BusinessAccountValidation),
    );
}
