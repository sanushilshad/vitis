use actix_web::web;

use crate::middlewares::{ProjectAccountValidation, ProjectPermissionValidation};

use super::{
    handlers::{
        fetch_project_req, list_project_req, project_permission_validation,
        register_project_account_req, user_project_association_req,
    },
    schemas::PermissionType,
};

pub fn project_routes(cfg: &mut web::ServiceConfig) {
    cfg.route("/register", web::post().to(register_project_account_req))
        .route("/fetch", web::post().to(fetch_project_req))
        .route("/list", web::get().to(list_project_req))
        .route("/permission", web::post().to(project_permission_validation))
        .route(
            "/user/association",
            web::post()
                .to(user_project_association_req)
                .wrap(ProjectPermissionValidation {
                    permission_list: vec![PermissionType::AssociateUserProject.to_string()],
                })
                .wrap(ProjectAccountValidation),
        );
}
