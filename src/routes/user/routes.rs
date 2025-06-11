use super::handlers::{
    authenticate_req, delete_user, fetch_user_req, reactivate_user_req, register_user_account_req,
    reset_password_req, send_otp_req, user_edit_req, user_list_req,
};
use crate::{
    middlewares::{RequireAuth, UserPermissionValidation},
    routes::project::schemas::PermissionType,
};

use actix_web::web;
pub fn user_routes(cfg: &mut web::ServiceConfig) {
    cfg.route("/register", web::post().to(register_user_account_req))
        .route(
            "/fetch",
            web::post().to(fetch_user_req).wrap(RequireAuth {
                allow_deleted_user: false,
            }),
        )
        .route("/authenticate", web::post().to(authenticate_req))
        .route("/otp/send", web::post().to(send_otp_req))
        .route(
            "/delete/{delete_type}",
            web::delete().to(delete_user).wrap(RequireAuth {
                allow_deleted_user: false,
            }),
        )
        .route(
            "/reactivate",
            web::patch().to(reactivate_user_req).wrap(RequireAuth {
                allow_deleted_user: true,
            }),
        )
        .route(
            "/list",
            web::post()
                .to(user_list_req)
                .wrap(UserPermissionValidation {
                    permission_list: vec![PermissionType::ListUsers.to_string()],
                })
                .wrap(RequireAuth {
                    allow_deleted_user: false,
                }),
        )
        .route(
            "/edit",
            web::post().to(user_edit_req).wrap(RequireAuth {
                allow_deleted_user: false,
            }),
        )
        .route(
            "/password/reset",
            web::post().to(reset_password_req).wrap(RequireAuth {
                allow_deleted_user: false,
            }),
        );
}
