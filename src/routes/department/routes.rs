use actix_web::web;

use crate::{
    middlewares::{
        BusinessPermissionValidation, DepartmentAccountValidation, DepartmentPermissionValidation,
    },
    schemas::PermissionType,
};

use super::handlers::{
    department_account_deletion_req, department_account_updation_req, fetch_department_req,
    list_department_req, register_department_account_req, user_department_association_req,
    user_department_deassociation_req,
};

// use crate::middlewares::{departmentAccountValidation, departmentPermissionValidation};

pub fn department_routes(cfg: &mut web::ServiceConfig) {
    cfg.route(
        "/register",
        web::post()
            .to(register_department_account_req)
            .wrap(BusinessPermissionValidation {
                permission_list: vec![PermissionType::CreateDepartment.to_string()],
            }),
    )
    .route("/fetch", web::post().to(fetch_department_req))
    .route("/list", web::get().to(list_department_req))
    .route(
        "/user/association",
        web::post()
            .to(user_department_association_req)
            .wrap(DepartmentPermissionValidation {
                permission_list: vec![PermissionType::AssociateUserDepartment.to_string()],
            })
            .wrap(DepartmentAccountValidation),
    )
    .route(
        "/user/disassociate",
        web::post()
            .to(user_department_deassociation_req)
            .wrap(DepartmentPermissionValidation {
                permission_list: vec![
                    PermissionType::DisassociateDepartment.to_string(),
                    PermissionType::DisassociateDepartmentSelf.to_string(),
                ],
            })
            .wrap(DepartmentAccountValidation),
    )
    .route(
        "/delete",
        web::delete()
            .to(department_account_deletion_req)
            .wrap(DepartmentPermissionValidation {
                permission_list: vec![PermissionType::DeleteDepartment.to_string()],
            })
            .wrap(DepartmentAccountValidation),
    )
    .route(
        "/update",
        web::patch()
            .to(department_account_updation_req)
            .wrap(DepartmentPermissionValidation {
                permission_list: vec![PermissionType::UpdateDepartment.to_string()],
            })
            .wrap(DepartmentAccountValidation),
    );
}
