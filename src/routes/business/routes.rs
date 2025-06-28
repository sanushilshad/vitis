use actix_web::web;

use crate::{
    middlewares::{
        BusinessAccountValidation, BusinessPermissionValidation, HeaderValidation, RequireAuth,
        UserPermissionValidation,
    },
    schemas::PermissionType,
};

use super::handlers::{
    business_permission_validation, business_user_invite_request, business_user_list_req,
    delete_business_user_invite, fetch_business_req, list_business_req, list_business_user_invite,
    register_business_account_req, user_business_association_req, verify_business_user_invite,
};

pub fn business_routes(cfg: &mut web::ServiceConfig) {
    cfg.route(
        "/register",
        web::post()
            .to(register_business_account_req)
            .wrap(RequireAuth {
                allow_deleted_user: false,
            })
            .wrap(HeaderValidation),
    )
    .route(
        "/fetch/{id}",
        web::get()
            .to(fetch_business_req)
            .wrap(RequireAuth {
                allow_deleted_user: false,
            })
            .wrap(HeaderValidation),
    )
    .route(
        "/list",
        web::get()
            .to(list_business_req)
            .wrap(UserPermissionValidation {
                permission_list: vec![
                    PermissionType::ListUserBusiness.to_string(),
                    PermissionType::ListUserBusinessSelf.to_string(),
                ],
            })
            .wrap(RequireAuth {
                allow_deleted_user: false,
            })
            .wrap(HeaderValidation),
    )
    .route(
        "/permission",
        web::post()
            .to(business_permission_validation)
            .wrap(HeaderValidation),
    )
    .route(
        "/user/association",
        web::post()
            .to(user_business_association_req)
            .wrap(BusinessPermissionValidation {
                permission_list: vec![PermissionType::AssociateUserBusiness.to_string()],
            })
            .wrap(BusinessAccountValidation)
            .wrap(RequireAuth {
                allow_deleted_user: false,
            })
            .wrap(HeaderValidation),
    )
    .route(
        "/user/list",
        web::post()
            .to(business_user_list_req)
            .wrap(BusinessAccountValidation)
            .wrap(RequireAuth {
                allow_deleted_user: false,
            })
            .wrap(HeaderValidation),
    )
    .route(
        "/invite/send",
        web::post()
            .to(business_user_invite_request)
            .wrap(BusinessPermissionValidation {
                permission_list: vec![PermissionType::SendBusinessInvite.to_string()],
            })
            .wrap(BusinessAccountValidation)
            .wrap(RequireAuth {
                allow_deleted_user: false,
            })
            .wrap(HeaderValidation),
    )
    .route(
        "/invite/delete/{id}",
        web::delete()
            .to(delete_business_user_invite)
            .wrap(BusinessPermissionValidation {
                permission_list: vec![PermissionType::SendBusinessInvite.to_string()],
            })
            .wrap(BusinessAccountValidation)
            .wrap(RequireAuth {
                allow_deleted_user: false,
            })
            .wrap(HeaderValidation),
    )
    .route(
        "/invite/list",
        web::post()
            .to(list_business_user_invite)
            .wrap(BusinessPermissionValidation {
                permission_list: vec![PermissionType::SendBusinessInvite.to_string()],
            })
            .wrap(BusinessAccountValidation)
            .wrap(RequireAuth {
                allow_deleted_user: false,
            })
            .wrap(HeaderValidation),
    )
    .route(
        "/invite/accept/{id}",
        web::post()
            .to(verify_business_user_invite)
            .wrap(RequireAuth {
                allow_deleted_user: false,
            }),
    );
}
