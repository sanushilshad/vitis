use crate::{
    middlewares::{
        BusinessAccountValidation, BusinessPermissionValidation, DepartmentAccountValidation,
        DepartmentPermissionValidation,
    },
    schemas::PermissionType,
};

use actix_web::web;

use super::handlers::{
    delete_business_role_req, list_business_role_req, list_role_permission_list_req,
    save_business_role_req,
};
pub fn role_routes(cfg: &mut web::ServiceConfig) {
    cfg.route(
        "/business/save",
        web::post()
            .to(save_business_role_req)
            .wrap(BusinessPermissionValidation {
                permission_list: vec![PermissionType::CreateBusinessRole.to_string()],
            })
            .wrap(BusinessAccountValidation),
    );
    cfg.route(
        "/business/list",
        web::get()
            .to(list_business_role_req)
            .wrap(BusinessPermissionValidation {
                permission_list: vec![PermissionType::CreateBusinessRole.to_string()],
            })
            .wrap(BusinessAccountValidation),
    );
    cfg.route(
        "/business/delete/{id}",
        web::delete()
            .to(delete_business_role_req)
            .wrap(BusinessPermissionValidation {
                permission_list: vec![PermissionType::CreateBusinessRole.to_string()],
            })
            .wrap(BusinessAccountValidation),
    );
    cfg.route(
        "/business-permission/list/{id}",
        web::get()
            .to(list_role_permission_list_req)
            .wrap(BusinessPermissionValidation {
                permission_list: vec![PermissionType::CreateBusinessRole.to_string()],
            })
            .wrap(BusinessAccountValidation),
    );
    cfg.route(
        "/department/save",
        web::post()
            .to(save_business_role_req)
            .wrap(DepartmentPermissionValidation {
                permission_list: vec![PermissionType::CreateDepartmentRole.to_string()],
            })
            .wrap(DepartmentAccountValidation)
            .wrap(BusinessAccountValidation),
    );
    cfg.route(
        "/department/list",
        web::get()
            .to(list_business_role_req)
            .wrap(DepartmentPermissionValidation {
                permission_list: vec![PermissionType::CreateDepartmentRole.to_string()],
            })
            .wrap(DepartmentAccountValidation)
            .wrap(BusinessAccountValidation),
    );
    cfg.route(
        "/department/delete/{id}",
        web::delete()
            .to(delete_business_role_req)
            .wrap(DepartmentPermissionValidation {
                permission_list: vec![PermissionType::CreateDepartmentRole.to_string()],
            })
            .wrap(DepartmentAccountValidation)
            .wrap(BusinessAccountValidation),
    );
}
