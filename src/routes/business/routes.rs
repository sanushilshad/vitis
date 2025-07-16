use actix_web::web;

use crate::{
    middlewares::{
        BusinessAccountValidation, BusinessPermissionValidation, UserPermissionValidation,
    },
    schemas::PermissionType,
};

use super::handlers::{
    business_account_deletion_req, business_account_updation_req, business_permission_validation,
    business_user_invite_request, business_user_list_req, delete_business_user_invite,
    fetch_business_req, list_business_req, list_business_user_invite,
    register_business_account_req, user_business_association_req, user_business_deassociation_req,
    verify_business_user_invite,
};

pub fn business_routes(cfg: &mut web::ServiceConfig) {
    cfg.route("/register", web::post().to(register_business_account_req))
        .route("/fetch/{id}", web::get().to(fetch_business_req))
        .route(
            "/list",
            web::get()
                .to(list_business_req)
                .wrap(UserPermissionValidation {
                    permission_list: vec![
                        PermissionType::ListUserBusiness.to_string(),
                        PermissionType::ListUserBusinessSelf.to_string(),
                    ],
                }),
        )
        .route(
            "/permission",
            web::post().to(business_permission_validation),
        )
        .route(
            "/user/associate",
            web::post()
                .to(user_business_association_req)
                .wrap(BusinessPermissionValidation {
                    permission_list: vec![PermissionType::AssociateUserBusiness.to_string()],
                })
                .wrap(BusinessAccountValidation),
        )
        .route(
            "/user/list",
            web::post()
                .to(business_user_list_req)
                .wrap(BusinessAccountValidation),
        )
        .route(
            "/invite/send",
            web::post()
                .to(business_user_invite_request)
                .wrap(BusinessPermissionValidation {
                    permission_list: vec![PermissionType::SendBusinessInvite.to_string()],
                })
                .wrap(BusinessAccountValidation),
        )
        .route(
            "/invite/delete/{id}",
            web::delete()
                .to(delete_business_user_invite)
                .wrap(BusinessPermissionValidation {
                    permission_list: vec![PermissionType::SendBusinessInvite.to_string()],
                })
                .wrap(BusinessAccountValidation),
        )
        .route(
            "/invite/list",
            web::get()
                .to(list_business_user_invite)
                .wrap(BusinessPermissionValidation {
                    permission_list: vec![PermissionType::SendBusinessInvite.to_string()],
                })
                .wrap(BusinessAccountValidation),
        )
        .route(
            "/invite/accept/{id}",
            web::post().to(verify_business_user_invite),
        )
        .route(
            "/user/disassociate",
            web::post()
                .to(user_business_deassociation_req)
                .wrap(BusinessPermissionValidation {
                    permission_list: vec![
                        PermissionType::DisassociateBusiness.to_string(),
                        PermissionType::DisassociateBusinessSelf.to_string(),
                    ],
                })
                .wrap(BusinessAccountValidation),
        )
        .route(
            "/delete",
            web::delete()
                .to(business_account_deletion_req)
                .wrap(BusinessPermissionValidation {
                    permission_list: vec![PermissionType::DeleteBusiness.to_string()],
                })
                .wrap(BusinessAccountValidation),
        )
        .route(
            "/update",
            web::patch()
                .to(business_account_updation_req)
                .wrap(BusinessPermissionValidation {
                    permission_list: vec![PermissionType::UpdateBusiness.to_string()],
                })
                .wrap(BusinessAccountValidation),
        );
}
