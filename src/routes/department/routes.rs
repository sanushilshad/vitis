use actix_web::web;

use crate::{
    middlewares::{
        BusinessPermissionValidation, DepartmentAccountValidation, DepartmentPermissionValidation,
    },
    schemas::PermissionType,
};

use super::handlers::{
    fetch_department_req, list_department_req, register_department_account_req,
    user_department_association_req,
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
    );
}
