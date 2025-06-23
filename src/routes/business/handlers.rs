use actix::Addr;
use actix_web::web;
use sqlx::PgPool;
use utoipa::TupleUnit;
use uuid::Uuid;

use crate::{
    errors::GenericError,
    routes::{
        user::{
            schemas::{MinimalUserAccount, UserAccount, UserRoleType},
            utils::{fetch_user_account_by_business_account, get_role},
        },
        web_socket::{schemas::ProcessType, utils::send_notification},
    },
    schemas::{AllowedPermission, GenericResponse, PermissionType, RequestMetaData},
    websocket_client::{Server, WebSocketActionType},
};

use super::{
    schemas::{
        BasicBusinessAccount, BusinessAccount, BusinessFetchRequest, BusinessPermissionRequest,
        BusinessUserAssociationRequest, CreateBusinessAccount,
    },
    utils::{
        associate_user_to_business, create_business_account, get_basic_business_accounts,
        get_basic_business_accounts_by_user_id, get_business_account,
        validate_user_business_permission,
    },
};

#[utoipa::path(
    post,
    description = "API for creating business accounts for a user. A single user can have multiple business accounts",
    summary = "business Account Registration API",
    path = "/business/register",
    tag = "business Account",
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
    tag = "business Account",
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
    tag = "business Account",
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
    tag = "business Account",
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
    path = "/user/assocation",
    tag = "business Account",
    description = "API for association of user with business account",
    summary = "Use business Account Association API",
    // request_body(content = BusinessAccountListReq, description = "Request Body"),
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
    path = "/user/list",
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
