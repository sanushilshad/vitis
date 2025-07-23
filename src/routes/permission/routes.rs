use crate::{
    middlewares::{
        BusinessAccountValidation, BusinessPermissionValidation, DepartmentAccountValidation,
        DepartmentPermissionValidation,
    },
    schemas::PermissionType,
};

use actix_web::web;

use super::handlers::{
    associate_permissions_to_business_role, associate_permissions_to_department_role,
    disassociate_permissions_to_business_role, disassociate_permissions_to_department_role,
    list_business_permissions, list_department_permissions,
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
            .to(associate_permissions_to_business_role)
            .wrap(BusinessPermissionValidation {
                permission_list: vec![PermissionType::CreateBusinessRole.to_string()],
            })
            .wrap(BusinessAccountValidation),
    );
    cfg.route(
        "/business-role/disassociate",
        web::delete()
            .to(disassociate_permissions_to_business_role)
            .wrap(BusinessPermissionValidation {
                permission_list: vec![PermissionType::CreateBusinessRole.to_string()],
            })
            .wrap(BusinessAccountValidation),
    );

    cfg.route(
        "/department/list",
        web::post()
            .to(list_department_permissions)
            .wrap(DepartmentPermissionValidation {
                permission_list: vec![PermissionType::CreateDepartmentRole.to_string()],
            })
            .wrap(DepartmentAccountValidation)
            .wrap(BusinessAccountValidation),
    );
    cfg.route(
        "/department-role/associate",
        web::post()
            .to(associate_permissions_to_department_role)
            .wrap(DepartmentPermissionValidation {
                permission_list: vec![PermissionType::CreateDepartmentRole.to_string()],
            })
            .wrap(DepartmentAccountValidation)
            .wrap(BusinessAccountValidation),
    );
    cfg.route(
        "/department-role/disassociate",
        web::delete()
            .to(disassociate_permissions_to_department_role)
            .wrap(DepartmentPermissionValidation {
                permission_list: vec![PermissionType::CreateDepartmentRole.to_string()],
            })
            .wrap(DepartmentAccountValidation)
            .wrap(BusinessAccountValidation),
    );
}
