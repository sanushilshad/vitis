use actix::Addr;
use actix_web::{HttpRequest, web};
use anyhow::Context;
use chrono::Utc;
use sqlx::PgPool;
use tera::Tera;
use utoipa::TupleUnit;
use uuid::Uuid;

use crate::{
    email_client::{GenericEmailService, SmtpEmailClient},
    errors::GenericError,
    routes::{
        setting::{
            schemas::{SettingKey, SettingsExt},
            utils::get_setting_value,
        },
        user::{
            schemas::{MinimalUserAccount, UserAccount},
            utils::{fetch_user_account_by_business_account, get_role, get_user},
        },
        web_socket::{schemas::ProcessType, utils::send_notification},
    },
    schemas::{AllowedPermission, GenericResponse, PermissionType, RequestMetaData},
    websocket_client::{Server, WebSocketActionType},
};

use super::{
    schemas::{
        BasicBusinessAccount, BusinessAccount, BusinessFetchRequest, BusinessInviteRequest,
        BusinessPermissionRequest, BusinessUserAssociationRequest, CreateBusinessAccount,
        UserBusinessInvitation,
    },
    utils::{
        associate_user_to_business, create_business_account, delete_invite_by_id,
        delete_user_business_relationship, fetch_business_invite, get_basic_business_accounts,
        get_basic_business_accounts_by_user_id, get_business_account, mark_invite_as_verified,
        save_business_invite_request, save_user_business_relation,
        validate_user_business_permission,
    },
};

#[utoipa::path(
    post,
    description = "API for creating business accounts for a user. A single user can have multiple business accounts",
    summary = "business Account Registration API",
    path = "/business/register",
    tag = "Business Account",
    request_body(content = CreateBusinessAccount, description = "Request Body"),
    responses(
        (status=200, description= "business Account created successfully", body= GenericResponse<TupleUnit>),
        (status=400, description= "Invalid Request body", body= GenericResponse<TupleUnit>),
        (status=401, description= "Invalid Token", body= GenericResponse<TupleUnit>),
	    (status=403, description= "Insufficient Previlege", body= GenericResponse<TupleUnit>),
	    (status=410, description= "Data not found", body= GenericResponse<TupleUnit>),
        (status=500, description= "Internal Server Error", body= GenericResponse<TupleUnit>)
    ),
    params(
        ("Authorization" = String, Header, description = "JWT token"),
        ("x-request-id" = String, Header, description = "Request id"),
        ("x-device-id" = String, Header, description = "Device id"),
      )
)]
#[tracing::instrument(
    err,
    name = "business Account Registration API",
    skip(pool, body),
    fields()
)]
pub async fn register_business_account_req(
    body: CreateBusinessAccount,
    pool: web::Data<PgPool>,
    meta_data: RequestMetaData,
    user: UserAccount,
) -> Result<web::Json<GenericResponse<()>>, GenericError> {
    create_business_account(&pool, &user, &body).await?;
    Ok(web::Json(GenericResponse::success(
        "Sucessfully Registered business Account.",
        (),
    )))
}

#[utoipa::path(
    get,
    path = "/business/fetch",
    tag = "Business Account",
    description = "API for fetching business account detail.",
    summary = "business Account Fetch API",
    request_body(content = BusinessFetchRequest, description = "Request Body"),
    responses(
        (status=200, description= "Sucessfully fetched business data.", body= GenericResponse<BusinessAccount>),
        (status=400, description= "Invalid Request body", body= GenericResponse<TupleUnit>),
        (status=401, description= "Invalid Token", body= GenericResponse<TupleUnit>),
	    (status=403, description= "Insufficient Previlege", body= GenericResponse<TupleUnit>),
	    (status=410, description= "Data not found", body= GenericResponse<TupleUnit>),
        (status=500, description= "Internal Server Error", body= GenericResponse<TupleUnit>)
    ),
    params(
        ("Authorization" = String, Header, description = "JWT token"),
        ("x-request-id" = String, Header, description = "Request id"),
        ("x-device-id" = String, Header, description = "Device id"),
        ("id" = String, Path, description = "Business ID"),
      )
)]
#[tracing::instrument(err, name = "fetch business detail", skip(db_pool), fields())]
pub async fn fetch_business_req(
    db_pool: web::Data<PgPool>,
    user_account: UserAccount,
    path: web::Path<Uuid>,
) -> Result<web::Json<GenericResponse<BusinessAccount>>, GenericError> {
    let business_id = path.into_inner();
    let business_account = get_business_account(&db_pool, user_account.id, business_id)
        .await
        .map_err(|e| GenericError::DatabaseError(e.to_string(), e))?
        .ok_or_else(|| {
            GenericError::ValidationError("business account does not exist.".to_string())
        })?;
    Ok(web::Json(GenericResponse::success(
        "Sucessfully fetched business data.",
        business_account,
    )))
}

#[utoipa::path(
    post,
    path = "/business/permission",
    tag = "Business Account",
    description = "API for checking the permission of a business.",
    summary = "business Account Permission API",
    request_body(content = BusinessPermissionRequest, description = "Request Body"),
    responses(
        (status=200, description= "Sucessfully verified permission.", body= GenericResponse<TupleUnit>),
        (status=400, description= "Invalid Request body", body= GenericResponse<TupleUnit>),
        (status=401, description= "Invalid Token", body= GenericResponse<TupleUnit>),
	    (status=403, description= "Insufficient Previlege", body= GenericResponse<TupleUnit>),
	    (status=410, description= "Data not found", body= GenericResponse<TupleUnit>),
        (status=500, description= "Internal Server Error", body= GenericResponse<TupleUnit>)
    ),
    params(
        ("Authorization" = String, Header, description = "JWT token"),
        ("x-request-id" = String, Header, description = "Request id"),
        ("x-device-id" = String, Header, description = "Device id"),
      )
)]
#[tracing::instrument(err, name = "business Permission", skip(db_pool), fields())]
pub async fn business_permission_validation(
    db_pool: web::Data<PgPool>,
    user_account: UserAccount,
    body: BusinessPermissionRequest,
) -> Result<web::Json<GenericResponse<Vec<String>>>, GenericError> {
    let permission_list = validate_user_business_permission(
        &db_pool,
        user_account.id,
        body.business_id,
        &body.action_list,
    )
    .await
    .map_err(|e| {
        GenericError::DatabaseError(
            "Something went wrong while fetching permission".to_owned(),
            e,
        )
    })?;

    if permission_list.is_empty() {
        return Err(GenericError::InsufficientPrevilegeError(
            "User doesn't have sufficient permission for the given action".to_owned(),
        ));
    }

    Ok(web::Json(GenericResponse::success(
        "Successfully validated permissions.",
        permission_list,
    )))
}

#[utoipa::path(
    get,
    path = "/business/list",
    tag = "Business Account",
    description = "API for listing all business account associated to a user",
    summary = "business Account List API",
    // request_body(content = BusinessAccountListReq, description = "Request Body"),
    responses(
        (status=200, description= "Sucessfully fetched business data.", body= GenericResponse<Vec<BasicBusinessAccount>>),
        (status=400, description= "Invalid Request body", body= GenericResponse<TupleUnit>),
        (status=401, description= "Invalid Token", body= GenericResponse<TupleUnit>),
	    (status=403, description= "Insufficient Previlege", body= GenericResponse<TupleUnit>),
	    (status=410, description= "Data not found", body= GenericResponse<TupleUnit>),
        (status=500, description= "Internal Server Error", body= GenericResponse<TupleUnit>)
    ),
    params(
        ("Authorization" = String, Header, description = "JWT token"),
        ("x-request-id" = String, Header, description = "Request id"),
        ("x-device-id" = String, Header, description = "Device id"),
      )
)]
#[tracing::instrument(err, name = "fetch business detail", skip(db_pool), fields())]
pub async fn list_business_req(
    db_pool: web::Data<PgPool>,
    user_account: UserAccount,
    permissions: AllowedPermission,
) -> Result<web::Json<GenericResponse<Vec<BasicBusinessAccount>>>, GenericError> {
    let business_obj = if !permissions
        .permission_list
        .contains(&PermissionType::ListUserBusiness.to_string())
    {
        get_basic_business_accounts_by_user_id(user_account.id, &db_pool).await?
    } else {
        get_basic_business_accounts(&db_pool)
            .await
            .map_err(|e| GenericError::DatabaseError(e.to_string(), e))?
    };
    Ok(web::Json(GenericResponse::success(
        "Sucessfully fetched all associated business accounts.",
        business_obj,
    )))
}

#[utoipa::path(
    post,
    path = "/business/user/associate",
    tag = "Business Account",
    description = "API for association of user with business account",
    summary = "Use business Account Association API",
    request_body(content = BusinessUserAssociationRequest, description = "Request Body"),
    responses(
        (status=200, description= "Sucessfully fetched business data.", body= GenericResponse<TupleUnit>),
        (status=400, description= "Invalid Request body", body= GenericResponse<TupleUnit>),
        (status=401, description= "Invalid Token", body= GenericResponse<TupleUnit>),
	    (status=403, description= "Insufficient Previlege", body= GenericResponse<TupleUnit>),
	    (status=410, description= "Data not found", body= GenericResponse<TupleUnit>),
        (status=500, description= "Internal Server Error", body= GenericResponse<TupleUnit>)
    ),
    params(
        ("Authorization" = String, Header, description = "JWT token"),
        ("x-request-id" = String, Header, description = "Request id"),
        ("x-device-id" = String, Header, description = "Device id"),
        ("x-business-id" = String, Header, description = "Business id"),
      )
)]
#[tracing::instrument(err, name = "user business association", skip(db_pool), fields())]
pub async fn user_business_association_req(
    req: BusinessUserAssociationRequest,
    db_pool: web::Data<PgPool>,
    user_account: UserAccount,
    business_account: BusinessAccount,
    websocket_srv: web::Data<Addr<Server>>,
) -> Result<web::Json<GenericResponse<()>>, GenericError> {
    let role_id = req.role_id.to_string();
    let role_obj_task = get_role(&db_pool, &role_id);

    let assocated_user_task = get_business_account(&db_pool, user_account.id, business_account.id);
    let assocating_user_task = get_business_account(&db_pool, req.user_id, business_account.id);
    let (role_obj_res, assocated_user_res, assocating_user_res) =
        tokio::join!(role_obj_task, assocated_user_task, assocating_user_task);

    assocated_user_res
        .map_err(|e| {
            GenericError::DatabaseError(
                "Something went wrong while fetching existing user-business association".to_owned(),
                e,
            )
        })?
        .ok_or_else(|| {
            GenericError::ValidationError("You are not associated to the business".to_owned())
        })?;

    let role_obj = role_obj_res
        .map_err(|e| {
            GenericError::DatabaseError("Something went wrong while fetching role".to_string(), e)
        })?
        .ok_or_else(|| GenericError::ValidationError("role does not exist.".to_string()))?;

    let assocating_user = assocating_user_res.map_err(|e| {
        GenericError::DatabaseError(
            "Something went wrong while fetching existing user-business association".to_owned(),
            e,
        )
    })?;

    if assocating_user.is_some() {
        return Err(GenericError::ValidationError(
            "User already associated with business".to_owned(),
        ));
    }
    associate_user_to_business(
        &db_pool,
        req.user_id,
        business_account.id,
        role_obj.id,
        user_account.id,
    )
    .await
    .map_err(|_| {
        GenericError::UnexpectedCustomError(
            "Something went wrong while associating user to business".to_owned(),
        )
    })?;
    // let msg: MessageToClient = MessageToClient::new(
    //     WebSocketActionType::UserBusinessAssociation,
    //     serde_json::to_value(WebSocketData {
    //         message: "Successfully associated user".to_string(),
    //     })
    //     .unwrap(),
    //     Some(user_account.id),
    //     None,
    //     None,
    // );
    // websocket_srv.do_send(msg);

    send_notification(
        &db_pool,
        &websocket_srv,
        WebSocketActionType::UserBusinessAssociation,
        ProcessType::Deferred,
        Some(req.user_id),
        format!(
            "{} Business is associated to you account by {}",
            business_account.display_name, user_account.display_name
        ),
        Some(business_account.id),
    )
    .await
    .map_err(|e| GenericError::UnexpectedCustomError(e.to_string()))?;
    Ok(web::Json(GenericResponse::success(
        "Sucessfully associated user with business account.",
        (),
    )))
}

#[utoipa::path(
    patch,
    path = "/business/user/list",
    tag = "Business Account",
    description = "API for listing users by business id",
    summary = "List User Accounts API by business id",

    responses(
        (status=200, description= "Successfully updated user.", body= GenericResponse<Vec<MinimalUserAccount>>),
        (status=400, description= "Invalid Request body", body= GenericResponse<TupleUnit>),
        (status=401, description= "Invalid Token", body= GenericResponse<TupleUnit>),
	    (status=403, description= "Insufficient Previlege", body= GenericResponse<TupleUnit>),
	    (status=410, description= "Data not found", body= GenericResponse<TupleUnit>),
        (status=500, description= "Internal Server Error", body= GenericResponse<TupleUnit>)
    ),
    params(
        ("x-request-id" = String, Header, description = "Request id"),
        ("x-device-id" = String, Header, description = "Device id"),
        ("Authorization" = String, Header, description = "JWT token"),
        ("x-business-id" = String, Header, description = "Business id"),
    )
)]
#[tracing::instrument(err, name = "List User Accounts By Business ID", skip(pool), fields())]
pub async fn business_user_list_req(
    pool: web::Data<PgPool>,
    user_account: UserAccount,
    business_account: BusinessAccount,
) -> Result<web::Json<GenericResponse<Vec<MinimalUserAccount>>>, GenericError> {
    let data = fetch_user_account_by_business_account(&pool, business_account.id)
        .await
        .map_err(|e| {
            tracing::error!("User list: {:?}", e);
            GenericError::DatabaseError("Failed to fetch users".to_string(), e)
        })?;

    Ok(web::Json(GenericResponse::success(
        "Successfully fetched users.",
        data,
    )))
}

#[utoipa::path(
    patch,
    path = "/business/invite/send",
    tag = "Business Account",
    description = "API for sending invite request to user for business association",
    summary = "List User Accounts API by business id",
    request_body(content = BusinessInviteRequest, description = "Request Body"),
    responses(
        (status=200, description= "Successfully updated user.", body= GenericResponse<TupleUnit>),
        (status=400, description= "Invalid Request body", body= GenericResponse<TupleUnit>),
        (status=401, description= "Invalid Token", body= GenericResponse<TupleUnit>),
	    (status=403, description= "Insufficient Previlege", body= GenericResponse<TupleUnit>),
	    (status=410, description= "Data not found", body= GenericResponse<TupleUnit>),
        (status=500, description= "Internal Server Error", body= GenericResponse<TupleUnit>)
    ),
    params(
        ("x-request-id" = String, Header, description = "Request id"),
        ("x-device-id" = String, Header, description = "Device id"),
        ("Authorization" = String, Header, description = "JWT token"),
        ("x-business-id" = String, Header, description = "Business id"),
    )
)]
#[tracing::instrument(
    err,
    name = "Send Business User Invite Request",
    skip(pool, email_client),
    fields()
)]
pub async fn business_user_invite_request(
    req: HttpRequest,
    body: BusinessInviteRequest,
    pool: web::Data<PgPool>,
    user_account: UserAccount,
    business_account: BusinessAccount,
    email_client: web::Data<SmtpEmailClient>,
) -> Result<web::Json<GenericResponse<()>>, GenericError> {
    let header = req
        .headers()
        .get("origin")
        .ok_or_else(|| GenericError::ValidationError("origin not found.".to_string()))?
        .to_str()
        .map_err(|_| GenericError::ValidationError("Invalid origin header".to_string()))?;
    let role_id = body.role_id.to_string();
    let setting_keys = vec![SettingKey::BusinessInviteRequestTemplate.to_string()];
    let role_task = get_role(&pool, &role_id);
    // let user_id = user_account.id.to_string();
    let existing_user_task = get_user(vec![&body.email.get()], &pool);
    let setting_task = get_setting_value(&pool, &setting_keys, None, None, true);
    let (role_res, setting_res, use_res) =
        tokio::join!(role_task, setting_task, existing_user_task);
    let role = role_res
        .map_err(|e| GenericError::DatabaseError("Failed to fetch role".to_string(), e))?
        .ok_or_else(|| GenericError::ValidationError("Role does not exist.".to_string()))?;
    let setting = setting_res.map_err(|e| GenericError::DatabaseError(e.to_string(), e))?;
    let existing_user =
        use_res.map_err(|e| GenericError::DatabaseError("Failed to fetch user".to_string(), e))?;
    if existing_user.is_some() {
        return Err(GenericError::ValidationError(
            "User already exists.".to_string(),
        ));
    }
    let invite_template = setting
        .get_setting(&SettingKey::BusinessInviteRequestTemplate.to_string())
        .ok_or_else(|| {
            GenericError::DataNotFound(format!(
                "Please set the {}",
                SettingKey::BusinessInviteRequestTemplate
            ))
        })?;

    let mut transaction = pool
        .begin()
        .await
        .context("Failed to acquire a Postgres connection from the pool")?;

    let id = save_business_invite_request(
        &mut transaction,
        user_account.id,
        business_account.id,
        role.id,
        &body.email,
    )
    .await
    .map_err(|e| GenericError::DatabaseError("Failed to save business invite".to_string(), e))?;

    //

    let context = tera::Context::from_serialize(
        serde_json::json!({ "link":  format!("{}/business/invite/accept/{}", header,  id) }),
    )
    .map_err(|_| GenericError::UnexpectedCustomError("Failed to render html".to_string()))?;
    let rendered_string = Tera::one_off(&invite_template, &context, true).map_err(|e| {
        tracing::error!("Error while rendering html error: {}", e);
        GenericError::UnexpectedCustomError(
            "Something went wrong while rendering the email html data".to_string(),
        )
    })?;
    email_client
        .send_html_email(
            &body.email,
            &None,
            "Invitation to vitis",
            rendered_string,
            None,
            None,
        )
        .await
        .map_err(|e| {
            tracing::error!("Failed to send mail: {:?}", e);
            GenericError::UnexpectedCustomError(
                "Something went wrong while sending email".to_string(),
            )
        })?;

    transaction
        .commit()
        .await
        .context("Failed to commit SQL transaction to store a new user account.")?;
    Ok(web::Json(GenericResponse::success(
        "Successfully send invite links.",
        (),
    )))
}

#[utoipa::path(
    patch,
    path = "/business/invite/list",
    tag = "Business Account",
    description = "API for listing invite request to user for business association",
    summary = "List Business User Invite Request API",

    responses(
        (status=200, description= "Successfully updated user.", body= GenericResponse<Vec<UserBusinessInvitation>>),
        (status=400, description= "Invalid Request body", body= GenericResponse<TupleUnit>),
        (status=401, description= "Invalid Token", body= GenericResponse<TupleUnit>),
	    (status=403, description= "Insufficient Previlege", body= GenericResponse<TupleUnit>),
	    (status=410, description= "Data not found", body= GenericResponse<TupleUnit>),
        (status=500, description= "Internal Server Error", body= GenericResponse<TupleUnit>)
    ),
    params(
        ("x-request-id" = String, Header, description = "Request id"),
        ("x-device-id" = String, Header, description = "Device id"),
        ("Authorization" = String, Header, description = "JWT token"),
        ("x-business-id" = String, Header, description = "Business id"),
    )
)]
#[tracing::instrument(err, name = "LIst Business User Invite Request", skip(pool), fields())]
pub async fn list_business_user_invite(
    req: HttpRequest,
    pool: web::Data<PgPool>,
    user_account: UserAccount,
    business_account: BusinessAccount,
) -> Result<web::Json<GenericResponse<Vec<UserBusinessInvitation>>>, GenericError> {
    let data = fetch_business_invite(
        &pool,
        None,
        Some(business_account.id),
        Some(user_account.id),
        None,
    )
    .await
    .map_err(|e| GenericError::DatabaseError("Failed to fetch business invite".to_string(), e))?;
    Ok(web::Json(GenericResponse::success(
        "Successfully fetched invite links.",
        data,
    )))
}

#[utoipa::path(
    post,
    path = "/business/invite/accept/{id}",
    tag = "Business Account",
    description = "API for accepting invite request to user for business association",
    summary = "Accept Business User Invite Request API",

    responses(
        (status=200, description= "Successfully updated user.", body= GenericResponse<TupleUnit>),
        (status=400, description= "Invalid Request body", body= GenericResponse<TupleUnit>),
        (status=401, description= "Invalid Token", body= GenericResponse<TupleUnit>),
	    (status=403, description= "Insufficient Previlege", body= GenericResponse<TupleUnit>),
	    (status=410, description= "Data not found", body= GenericResponse<TupleUnit>),
        (status=500, description= "Internal Server Error", body= GenericResponse<TupleUnit>)
    ),
    params(
        ("x-request-id" = String, Header, description = "Request id"),
        ("x-device-id" = String, Header, description = "Device id"),
        ("Authorization" = String, Header, description = "JWT token"),
        ("x-business-id" = String, Header, description = "Business id"),
        ("id" = String, Path, description = "Invite ID"),
    )
)]
#[tracing::instrument(err, name = "LIst Business User Invite Request", skip(pool), fields())]
pub async fn verify_business_user_invite(
    path: web::Path<Uuid>,
    pool: web::Data<PgPool>,
    user_account: UserAccount,
) -> Result<web::Json<GenericResponse<()>>, GenericError> {
    let id = path.into_inner();
    let data_list = fetch_business_invite(&pool, None, None, None, Some(vec![id]))
        .await
        .map_err(|e| {
            GenericError::DatabaseError("Failed to fetch business invite".to_string(), e)
        })?;
    if let Some(invite) = data_list.first() {
        if invite.verified {
            return Err(GenericError::ValidationError(
                "Invite is already accepted.".to_string(),
            ));
        }
        let mut transaction = pool
            .begin()
            .await
            .context("Failed to acquire a Postgres connection from the pool")?;
        save_user_business_relation(
            &mut transaction,
            user_account.id,
            invite.business_id,
            invite.role_id,
        )
        .await
        .map_err(|e| {
            GenericError::DatabaseError(
                "Failed to save user business relation".to_string(),
                e.into(),
            )
        })?;

        mark_invite_as_verified(&mut transaction, invite.id, user_account.id, Utc::now())
            .await
            .map_err(|e| {
                GenericError::DatabaseError(
                    "Failed to mark invite as verified".to_string(),
                    e.into(),
                )
            })?;
        transaction
            .commit()
            .await
            .context("Failed to commit SQL transaction to store a new user account.")?;
    } else {
        return Err(GenericError::ValidationError(
            "invalid invitation.".to_string(),
        ));
    }
    Ok(web::Json(GenericResponse::success(
        "Successfully associated user to business.",
        (),
    )))
}

#[utoipa::path(
    delete,
    path = "/business/invite/delete/{id}",
    tag = "Business Account",
    description = "API for deleting invite request to user for business association",
    summary = "Accept Business User Invite Request API",

    responses(
        (status=200, description= "Successfully updated user.", body= GenericResponse<TupleUnit>),
        (status=400, description= "Invalid Request body", body= GenericResponse<TupleUnit>),
        (status=401, description= "Invalid Token", body= GenericResponse<TupleUnit>),
	    (status=403, description= "Insufficient Previlege", body= GenericResponse<TupleUnit>),
	    (status=410, description= "Data not found", body= GenericResponse<TupleUnit>),
        (status=500, description= "Internal Server Error", body= GenericResponse<TupleUnit>)
    ),
    params(
        ("x-request-id" = String, Header, description = "Request id"),
        ("x-device-id" = String, Header, description = "Device id"),
        ("Authorization" = String, Header, description = "JWT token"),
        ("x-business-id" = String, Header, description = "Business id"),
        ("id" = String, Path, description = "Invite ID"),

    )
)]
#[tracing::instrument(err, name = "LIst Business User Invite Request", skip(pool), fields())]
pub async fn delete_business_user_invite(
    path: web::Path<Uuid>,
    pool: web::Data<PgPool>,
    user_account: UserAccount,
) -> Result<web::Json<GenericResponse<()>>, GenericError> {
    let id = path.into_inner();
    let data_list = fetch_business_invite(&pool, None, None, Some(user_account.id), Some(vec![id]))
        .await
        .map_err(|e| {
            GenericError::DatabaseError("Failed to fetch business invite".to_string(), e)
        })?;
    if let Some(invite) = data_list.first() {
        if invite.verified {
            return Err(GenericError::ValidationError(
                "Invite is already accepted.".to_string(),
            ));
        }
        delete_invite_by_id(&pool, invite.id).await.map_err(|e| {
            GenericError::DatabaseError("Failed to delete business invite".to_string(), e.into())
        })?;
    } else {
        return Err(GenericError::ValidationError(
            "Invite invitation ID.".to_string(),
        ));
    }
    Ok(web::Json(GenericResponse::success(
        "Successfully deleted invite.",
        (),
    )))
}

#[utoipa::path(
    post,
    path = "/business/user/disassociate",
    tag = "Business Account",
    description = "API for disassociating user froms business account",
    summary = "Use business Account Disassociation API",
    responses(
        (status=200, description= "Sucessfully fetched business data.", body= GenericResponse<TupleUnit>),
        (status=400, description= "Invalid Request body", body= GenericResponse<TupleUnit>),
        (status=401, description= "Invalid Token", body= GenericResponse<TupleUnit>),
	    (status=403, description= "Insufficient Previlege", body= GenericResponse<TupleUnit>),
	    (status=410, description= "Data not found", body= GenericResponse<TupleUnit>),
        (status=500, description= "Internal Server Error", body= GenericResponse<TupleUnit>)
    ),
    params(
        ("Authorization" = String, Header, description = "JWT token"),
        ("x-request-id" = String, Header, description = "Request id"),
        ("x-device-id" = String, Header, description = "Device id"),
        ("x-business-id" = String, Header, description = "Business id"),
      )
)]
#[tracing::instrument(err, name = "user business disassociation", skip(pool), fields())]
pub async fn user_business_deassociation_req(
    pool: web::Data<PgPool>,
    user_account: UserAccount,
    business_account: BusinessAccount,
) -> Result<web::Json<GenericResponse<()>>, GenericError> {
    delete_user_business_relationship(&pool, user_account.id, business_account.id)
        .await
        .map_err(|e| {
            GenericError::DatabaseError(
                "Something went wrong while disassociating user from business".to_owned(),
                e,
            )
        })?;
    Ok(web::Json(GenericResponse::success(
        "Sucessfully disassociated user from business account.",
        (),
    )))
}
