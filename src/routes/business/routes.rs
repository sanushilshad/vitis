use actix_web::web;

use crate::{
    middlewares::{BusinessAccountValidation, BusinessPermissionValidation},
    schemas::PermissionType,
};

use super::handlers::{
    business_permission_validation, fetch_business_req, list_business_req,
    register_business_account_req, user_business_association_req,
};

pub fn business_routes(cfg: &mut web::ServiceConfig) {
    cfg.route("/register", web::post().to(register_business_account_req))
        .route("/fetch/{id}", web::get().to(fetch_business_req))
        .route("/list", web::get().to(list_business_req))
        .route(
            "/permission",
            web::post().to(business_permission_validation),
        )
        .route(
            "/user/association",
            web::post()
                .to(user_business_association_req)
                .wrap(BusinessPermissionValidation {
                    permission_list: vec![PermissionType::AssociateUserBusiness.to_string()],
                })
                .wrap(BusinessAccountValidation),
        );
}
