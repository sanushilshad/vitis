use actix_web::web;

use crate::{middlewares::BusinessPermissionValidation, schemas::PermissionType};

use super::handlers::{
    create_leave_req, create_leave_user_association_req, delete_leave_user_association_req,
    leave_group_create_req, leave_group_delete_req, leave_group_list_req, leave_period_create_req,
    leave_period_delete_req, leave_period_list_req, leave_request_deletion_req,
    leave_request_fetch_req, leave_type_create_req, leave_type_delete_req, leave_type_list_req,
    list_leave_user_association_req, update_leave_status_req,
};

pub fn leave_routes(cfg: &mut web::ServiceConfig) {
    cfg.route(
        "/type/create",
        web::post()
            .to(leave_type_create_req)
            .wrap(BusinessPermissionValidation {
                permission_list: vec![PermissionType::CreateLeaveType.to_string()],
            }),
    );
    cfg.route(
        "/type/delete/{id}",
        web::delete()
            .to(leave_type_delete_req)
            .wrap(BusinessPermissionValidation {
                permission_list: vec![PermissionType::CreateLeaveType.to_string()],
            }),
    );
    cfg.route(
        "/type/list",
        web::post()
            .to(leave_type_list_req)
            .wrap(BusinessPermissionValidation {
                permission_list: vec![PermissionType::CreateLeaveType.to_string()],
            }),
    );

    cfg.route(
        "/group/create",
        web::post()
            .to(leave_group_create_req)
            .wrap(BusinessPermissionValidation {
                permission_list: vec![PermissionType::CreateLeaveType.to_string()],
            }),
    );
    cfg.route(
        "/group/delete/{id}",
        web::delete()
            .to(leave_group_delete_req)
            .wrap(BusinessPermissionValidation {
                permission_list: vec![PermissionType::CreateLeaveType.to_string()],
            }),
    );

    cfg.route("/group/list", web::post().to(leave_group_list_req));
    cfg.route(
        "/user/association/save",
        web::post()
            .to(create_leave_user_association_req)
            .wrap(BusinessPermissionValidation {
                permission_list: vec![PermissionType::CreateLeaveType.to_string()],
            }),
    );
    cfg.route(
        "/user/association/delete/{id}",
        web::delete()
            .to(delete_leave_user_association_req)
            .wrap(BusinessPermissionValidation {
                permission_list: vec![PermissionType::CreateLeaveType.to_string()],
            }),
    );
    cfg.route(
        "/user/association/list",
        web::post()
            .to(list_leave_user_association_req)
            .wrap(BusinessPermissionValidation {
                permission_list: vec![PermissionType::CreateLeaveType.to_string()],
            }),
    );
    cfg.route(
        "/request/create",
        web::post()
            .to(create_leave_req)
            .wrap(BusinessPermissionValidation {
                permission_list: vec![
                    PermissionType::CreateLeaveRequestSelf.to_string(),
                    PermissionType::CreateLeaveRequest.to_string(),
                ],
            }),
    );
    cfg.route(
        "/request/delete/{id}",
        web::delete()
            .to(leave_request_deletion_req)
            .wrap(BusinessPermissionValidation {
                permission_list: vec![
                    PermissionType::CreateLeaveRequestSelf.to_string(),
                    PermissionType::CreateLeaveRequest.to_string(),
                ],
            }),
    );
    cfg.route(
        "/request/list",
        web::post()
            .to(leave_request_fetch_req)
            .wrap(BusinessPermissionValidation {
                permission_list: vec![
                    PermissionType::ListLeaveRequestSelf.to_string(),
                    PermissionType::ListLeaveRequest.to_string(), // PermissionType::CreateLeaveRequest.to_string(),
                ],
            }),
    );
    cfg.route(
        "/request/status/update",
        web::patch()
            .to(update_leave_status_req)
            .wrap(BusinessPermissionValidation {
                permission_list: vec![
                    PermissionType::ApproveLeaveRequest.to_string(),
                    PermissionType::UpdateLeaveRequestStatus.to_string(), // PermissionType::CreateLeaveRequest.to_string(),
                ],
            }),
    );

    cfg.route(
        "/period/create",
        web::post()
            .to(leave_period_create_req)
            .wrap(BusinessPermissionValidation {
                permission_list: vec![PermissionType::CreateLeaveType.to_string()],
            }),
    );
    cfg.route(
        "/period/delete/{id}",
        web::delete()
            .to(leave_period_delete_req)
            .wrap(BusinessPermissionValidation {
                permission_list: vec![PermissionType::CreateLeaveType.to_string()],
            }),
    );

    cfg.route(
        "/period/list",
        web::post()
            .to(leave_period_list_req)
            .wrap(BusinessPermissionValidation {
                permission_list: vec![PermissionType::CreateLeaveType.to_string()],
            }),
    );
}
