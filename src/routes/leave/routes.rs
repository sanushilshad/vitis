use actix_web::web;

use crate::{middlewares::UserPermissionValidation, routes::project::schemas::PermissionType};

use super::handlers::{
    create_leave_req, leave_request_deletion_req, leave_request_fetch_req, update_leave_status_req,
};

pub fn leave_routes(cfg: &mut web::ServiceConfig) {
    cfg.route(
        "/create",
        web::post()
            .to(create_leave_req)
            .wrap(UserPermissionValidation {
                permission_list: vec![
                    PermissionType::CreateLeaveRequestSelf.to_string(),
                    PermissionType::CreateLeaveRequest.to_string(),
                ],
            }),
    );
    cfg.route(
        "/status/update",
        web::patch()
            .to(update_leave_status_req)
            .wrap(UserPermissionValidation {
                permission_list: vec![
                    PermissionType::ApproveLeaveRequest.to_string(),
                    PermissionType::UpdateLeaveRequestStatus.to_string(), // PermissionType::CreateLeaveRequest.to_string(),
                ],
            }),
    );
    cfg.route("/delete/{id}", web::delete().to(leave_request_deletion_req));
    cfg.route("/fetch", web::post().to(leave_request_fetch_req));
}
