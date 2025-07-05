use std::collections::HashSet;

use crate::pulsar_client::{PulsarClient, PulsarTopic, SchedulerMessageData};
use crate::routes::business::schemas::BusinessAccount;
use crate::routes::web_socket::schemas::ProcessType;
use crate::routes::web_socket::utils::send_notification;
use crate::websocket_client::Server;
use crate::{
    configuration::EmailClientConfig,
    email_client::{GenericEmailService, SmtpEmailClient},
    errors::GenericError,
    routes::{
        setting::{
            schemas::{SettingKey, SettingsExt},
            utils::get_setting_value,
        },
        user::{
            schemas::{UserAccount, VectorType},
            utils::get_user,
        },
    },
    schemas::{AllowedPermission, GenericResponse, PermissionType},
    utils::to_title_case,
    websocket_client::WebSocketActionType,
};
use actix::Addr;
use actix_web::web;
use anyhow::Context;
use bigdecimal::BigDecimal;

use chrono_tz::Tz;
use secrecy::SecretString;
use sqlx::PgPool;
use tera::{Context as TeraContext, Tera};
use tokio::join;
use utoipa::TupleUnit;
use uuid::Uuid;

use super::schemas::{
    CreateLeaveRequest, FetchLeaveQuery, FetchLeaveRequest, FetchLeaveType,
    LeavePeriodCreationRequest, LeavePeriodData, LeavePeriodFetchRequest, LeaveRequestData,
    LeaveRequestEmailContext, LeaveRequestStatusEmailContext, UpdateLeaveStatusRequest,
};
use super::schemas::{
    CreateLeaveUserAssociationRequest, LeaveGroup, LeaveGroupCreationRequest, LeaveStatus,
    LeaveTypeCreationRequest, LeaveTypeData, LeaveTypeFetchRequest,
    ListLeaveUserAssociationRequest, UserLeave,
};
use super::utils::{
    delete_leave, delete_leave_group, delete_leave_period, delete_leave_type, delete_user_leave,
    fetch_user_leaves, get_leave_group, get_leave_period, get_leave_type, get_leaves,
    leave_group_create_validation, leave_type_create_validation, save_leave_group,
    save_leave_period, save_leave_request, save_leave_type, save_user_leave,
    update_leave_request_status, update_user_leave_count, validate_leave_request_creation,
    validate_leave_status_update,
};

#[utoipa::path(
    post,
    description = "API for making listing leave request",
    tag = "Leave",
    summary = "Leave Request Fetch API",
    path = "/leave/request/list",
    request_body(content = FetchLeaveRequest, description = "Request Body"),
    responses(
        (status=200, description= "project Account created successfully", body= GenericResponse<Vec<LeaveRequestData>>),
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
        // ("id" = String, Path, description = "Leave ID"),
      )
)]
#[tracing::instrument(err, name = "Leave Request listing request", skip(pool), fields())]
pub async fn leave_request_fetch_req(
    pool: web::Data<PgPool>,
    req: FetchLeaveRequest,
    user: UserAccount,
    mail_config: web::Data<EmailClientConfig>,
    permissions: AllowedPermission,
) -> Result<web::Json<GenericResponse<Vec<LeaveRequestData>>>, GenericError> {
    let setting_key_list = vec![SettingKey::TimeZone.to_string()];
    let setting_list = get_setting_value(&pool, &setting_key_list, None, Some(user.id), false)
        .await
        .map_err(|e| GenericError::DatabaseError(e.to_string(), e))?;
    let timezone = setting_list
        .get_setting(&SettingKey::TimeZone.to_string())
        .ok_or_else(|| {
            GenericError::DataNotFound(format!("Please set the {}", SettingKey::TimeZone))
        })?;
    let sender_id = if req.user_id.is_some()
        && !permissions
            .permission_list
            .contains(&PermissionType::CreateLeaveRequest.to_string())
    {
        req.user_id
    } else if req.action_type == FetchLeaveType::Sender {
        Some(user.id)
    } else {
        None
    };

    let receiver_id = if req.action_type == FetchLeaveType::Receiver {
        Some(user.id)
    } else {
        None
    };
    let tz: Tz = timezone
        .parse()
        .map_err(|_| GenericError::DataNotFound("please set the timezone".to_string()))?;
    let filter_query = FetchLeaveQuery::builder()
        .with_leave_id(req.id)
        .with_sender_id(sender_id)
        .with_recevier_id(receiver_id)
        .with_tz(Some(&tz))
        .with_start_date(req.start_date.as_ref())
        .with_end_date(req.end_date.as_ref())
        .with_limit(Some(req.limit))
        .with_offset(Some(req.offset));
    let leave = get_leaves(&pool, &filter_query).await.map_err(|e| {
        GenericError::DatabaseError(
            "Something went wrong while fetching leave data".to_string(),
            e,
        )
    })?;
    Ok(web::Json(GenericResponse::success(
        "Sucessfully fetched leave request",
        leave,
    )))
}

#[utoipa::path(
    post,
    description = "API for creating leave type",
    tag = "Leave",
    summary = "Leave Type Creation API",
    path = "/leave/type/create",
    request_body(content = LeaveTypeCreationRequest, description = "Request Body"),
    responses(
        (status=200, description= "project Account created successfully", body= GenericResponse<TupleUnit>),
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
        ("x-business-id" = String, Header, description = "id of business_account"),
        // ("id" = String, Path, description = "Leave ID"),
      )
)]
#[tracing::instrument(err, name = "Leave type create request", skip(pool), fields())]
pub async fn leave_type_create_req(
    pool: web::Data<PgPool>,
    req: LeaveTypeCreationRequest,
    user: UserAccount,
    business_account: BusinessAccount,
) -> Result<web::Json<GenericResponse<()>>, GenericError> {
    leave_type_create_validation(&pool, &req, business_account.id).await?;
    let mut transaction = pool
        .begin()
        .await
        .context("Failed to acquire a Postgres connection from the pool")?;
    save_leave_type(
        &mut transaction,
        &req.data,
        user.id,
        user.id,
        business_account.id,
    )
    .await
    .map_err(|e| {
        GenericError::DatabaseError(
            "Something went wrong while saving leave type".to_string(),
            e,
        )
    })?;

    transaction
        .commit()
        .await
        .context("Failed to commit SQL transaction to store a save leave type.")?;
    Ok(web::Json(GenericResponse::success(
        "Sucessfully saved leave type",
        (),
    )))
}

#[utoipa::path(
    post,
    description = "API for listing leave type",
    tag = "Leave",
    summary = "Leave Type List API",
    path = "/leave/type/list",
    request_body(content = LeaveTypeFetchRequest, description = "Request Body"),
    responses(
        (status=200, description= "project Account created successfully", body= GenericResponse<Vec<LeaveTypeData>>),
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
        ("x-business-id" = String, Header, description = "id of business_account"),
        // ("id" = String, Path, description = "Leave ID"),
      )
)]
#[tracing::instrument(err, name = "Leave type list request", skip(pool), fields())]
pub async fn leave_type_list_req(
    pool: web::Data<PgPool>,
    req: LeaveTypeFetchRequest,
    user: UserAccount,
    business_account: BusinessAccount,
) -> Result<web::Json<GenericResponse<Vec<LeaveTypeData>>>, GenericError> {
    let leave_type_list = get_leave_type(&pool, business_account.id, None, None, req.query)
        .await
        .map_err(|e| {
            GenericError::DatabaseError(
                "Something went wrong while fetching leave type".to_string(),
                e,
            )
        })?;

    Ok(web::Json(GenericResponse::success(
        "Sucessfully fetched leave type",
        leave_type_list,
    )))
}

#[utoipa::path(
    delete,
    description = "API for deleting leave type",
    tag = "Leave",
    summary = "Leave Type Delete API",
    path = "/leave/type/delete",
    // request_body(content = FetchLeaveRequest, description = "Request Body"),
    responses(
        (status=200, description= "project Account created successfully", body= GenericResponse<TupleUnit>),
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
        ("x-business-id" = String, Header, description = "id of business_account"),
        ("id" = String, Path, description = "Leave Type ID"),
      )
)]
#[tracing::instrument(err, name = "Leave type delete request", skip(pool), fields())]
pub async fn leave_type_delete_req(
    path: web::Path<Uuid>,
    pool: web::Data<PgPool>,
    user: UserAccount,
    business_account: BusinessAccount,
) -> Result<web::Json<GenericResponse<()>>, GenericError> {
    let leave_type_id = path.into_inner();
    let leave_type_list = get_leave_type(
        &pool,
        business_account.id,
        Some(vec![leave_type_id]),
        None,
        None,
    )
    .await
    .map_err(|e| {
        GenericError::DatabaseError(
            "Something went wrong while deleting leave type".to_string(),
            e,
        )
    })?;
    leave_type_list
        .first()
        .ok_or_else(|| GenericError::DataNotFound("Invalid Leave Type id".to_string()))?;

    delete_leave_type(&pool, leave_type_id).await.map_err(|e| {
        GenericError::DatabaseError(
            "Something went wrong while deleting leave type".to_string(),
            e,
        )
    })?;

    Ok(web::Json(GenericResponse::success(
        "Sucessfully deleted leave type",
        (),
    )))
}

#[utoipa::path(
    post,
    description = "API for creating leave group",
    tag = "Leave",
    summary = "Leave Group Creation API",
    path = "/leave/group/create",
    request_body(content = LeaveGroupCreationRequest, description = "Request Body"),
    responses(
        (status=200, description= "project Account created successfully", body= GenericResponse<TupleUnit>),
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
        ("x-business-id" = String, Header, description = "id of business_account"),
        // ("id" = String, Path, description = "Leave ID"),
      )
)]
#[tracing::instrument(err, name = "Leave group create request", skip(pool), fields())]
pub async fn leave_group_create_req(
    pool: web::Data<PgPool>,
    req: LeaveGroupCreationRequest,
    user: UserAccount,
    business_account: BusinessAccount,
) -> Result<web::Json<GenericResponse<()>>, GenericError> {
    leave_group_create_validation(&pool, &req, business_account.id).await?;
    save_leave_group(&pool, &req, business_account.id, user.id)
        .await
        .map_err(|e| {
            GenericError::DatabaseError(
                "Something went wrong while saving leave group".to_string(),
                e,
            )
        })?;
    Ok(web::Json(GenericResponse::success(
        "Sucessfully saved leave group",
        (),
    )))
}

#[utoipa::path(
    delete,
    description = "API for deleting leave group",
    tag = "Leave",
    summary = "Leave Group Delete API",
    path = "/leave/group/delete",
    // request_body(content = FetchLeaveRequest, description = "Request Body"),
    responses(
        (status=200, description= "project Account created successfully", body= GenericResponse<TupleUnit>),
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
        ("x-business-id" = String, Header, description = "id of business_account"),
        ("id" = String, Path, description = "Leave Group ID"),
      )
)]
#[tracing::instrument(err, name = "Leave group delete request", skip(pool), fields())]
pub async fn leave_group_delete_req(
    path: web::Path<Uuid>,
    pool: web::Data<PgPool>,
    user: UserAccount,
    business_account: BusinessAccount,
) -> Result<web::Json<GenericResponse<()>>, GenericError> {
    let leave_group_id = path.into_inner();
    let leave_group_list = get_leave_group(
        &pool,
        business_account.id,
        Some(&[leave_group_id]),
        None,
        None,
        None,
    )
    .await
    .map_err(|e| {
        GenericError::DatabaseError(
            "Something went wrong while fetching leave group".to_string(),
            e,
        )
    })?;
    leave_group_list
        .first()
        .ok_or_else(|| GenericError::DataNotFound("Invalid Leave Group id".to_string()))?;

    delete_leave_group(&pool, leave_group_id)
        .await
        .map_err(|e| {
            GenericError::DatabaseError(
                "Something went wrong while deleting leave group".to_string(),
                e,
            )
        })?;

    Ok(web::Json(GenericResponse::success(
        "Sucessfully deleted leave group",
        (),
    )))
}

#[utoipa::path(
    post,
    description = "API for listing leave group",
    tag = "Leave",
    summary = "Leave Group List API",
    path = "/leave/group/list",
    request_body(content = FetchLeaveRequest, description = "Request Body"),
    responses(
        (status=200, description= "project Account created successfully", body= GenericResponse<Vec<LeaveGroup>>),
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
        ("x-business-id" = String, Header, description = "id of business_account"),
        // ("id" = String, Path, description = "Leave ID"),
      )
)]
#[tracing::instrument(err, name = "Leave group list request", skip(pool), fields())]
pub async fn leave_group_list_req(
    pool: web::Data<PgPool>,
    req: LeaveTypeFetchRequest,
    user: UserAccount,
    business_account: BusinessAccount,
) -> Result<web::Json<GenericResponse<Vec<LeaveGroup>>>, GenericError> {
    let leave_group_list = get_leave_group(&pool, business_account.id, None, req.query, None, None)
        .await
        .map_err(|e| {
            GenericError::DatabaseError(
                "Something went wrong while fetching leave groups".to_string(),
                e,
            )
        })?;

    Ok(web::Json(GenericResponse::success(
        "Sucessfully fetched leave groups",
        leave_group_list,
    )))
}

#[utoipa::path(
    post,
    description = "API for associating user to leave",
    tag = "Leave",
    summary = "Leave User Association API",
    path = "/leave/user/association/save",
    request_body(content = CreateLeaveUserAssociationRequest, description = "Request Body"),
    responses(
        (status=200, description= "project Account created successfully", body= GenericResponse<TupleUnit>),
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
        ("x-business-id" = String, Header, description = "id of business_account"),
      )
)]
#[tracing::instrument(err, name = "User Leave Association request", skip(pool), fields())]
pub async fn create_leave_user_association_req(
    pool: web::Data<PgPool>,
    data: CreateLeaveUserAssociationRequest,

    user: UserAccount,
    business_account: BusinessAccount,
) -> Result<web::Json<GenericResponse<()>>, GenericError> {
    let group_id_list = vec![data.group_id];
    let leave_type_set: HashSet<Uuid> = data.data.iter().map(|x| x.type_id).collect();
    let leave_type_list: Vec<Uuid> = leave_type_set.iter().copied().collect();

    let (leave_group_res, leave_type_res) = join!(
        get_leave_group(
            &pool,
            business_account.id,
            Some(&group_id_list),
            None,
            None,
            None
        ),
        get_leave_type(
            &pool,
            business_account.id,
            Some(leave_type_list.to_vec()),
            None,
            None
        ),
    );
    let leave_group_list =
        leave_group_res.map_err(|e| GenericError::DatabaseError(e.to_string(), e))?;
    let leave_group = leave_group_list.first().ok_or(GenericError::DataNotFound(
        "Leave group not found.".to_string(),
    ))?;
    let leave_type = leave_type_res.map_err(|e| GenericError::DatabaseError(e.to_string(), e))?;
    let allowed_leave_type_set: HashSet<Uuid> = leave_type.iter().map(|x| x.id).collect();
    if !allowed_leave_type_set.is_superset(&leave_type_set) {
        return Err(GenericError::DataNotFound(
            "Leave type/s not found for given business".to_string(),
        ));
    }
    save_user_leave(&pool, &data.data, user.id, leave_group.id)
        .await
        .map_err(|e| {
            GenericError::DatabaseError(
                "Something went wrong while saving user leave association".to_string(),
                e,
            )
        })?;

    Ok(web::Json(GenericResponse::success(
        "Sucessfully associated user to leave",
        (),
    )))
}

#[utoipa::path(
    post,
    description = "API for listing associating user to leave",
    tag = "Leave",
    summary = "List Leave User Association API",
    path = "/leave/user/association/list",
    request_body(content = ListLeaveUserAssociationRequest, description = "Request Body"),
    responses(
        (status=200, description= "project Account created successfully", body= GenericResponse<Vec<UserLeave>>),
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
        ("x-business-id" = String, Header, description = "id of business_account"),
      )
)]
#[tracing::instrument(
    err,
    name = "List User Leave Association request",
    skip(pool),
    fields()
)]
pub async fn list_leave_user_association_req(
    req: ListLeaveUserAssociationRequest,
    pool: web::Data<PgPool>,
    user: UserAccount,
    business_account: BusinessAccount,
    permissions: AllowedPermission,
) -> Result<web::Json<GenericResponse<Vec<UserLeave>>>, GenericError> {
    let user_id = if !permissions
        .permission_list
        .contains(&PermissionType::ListUserLeave.to_string())
    {
        req.user_id.unwrap_or(user.id)
    } else {
        user.id
    };
    let data = fetch_user_leaves(
        &pool,
        business_account.id,
        user_id,
        Some(req.group_id),
        None,
    )
    .await
    .map_err(|e| {
        GenericError::DatabaseError(
            "Something went wrong while fetching user leave association".to_string(),
            e,
        )
    })?;
    Ok(web::Json(GenericResponse::success(
        "Sucessfully fetched user leaves",
        data,
    )))
}

#[utoipa::path(
    delete,
    description = "API for delete associated user to leave",
    tag = "Leave",
    summary = "Delete Leave User Association API",
    path = "/leave/user/association/delete",
    responses(
        (status=200, description= "project Account created successfully", body= GenericResponse<TupleUnit>),
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
        ("x-business-id" = String, Header, description = "id of business_account"),
        ("id" = String, Path, description = "Leave User ID"),
      )
)]
#[tracing::instrument(
    err,
    name = "Delete User Leave Association request",
    skip(pool),
    fields()
)]
pub async fn delete_leave_user_association_req(
    path: web::Path<Uuid>,
    pool: web::Data<PgPool>,
    user: UserAccount,
    business_account: BusinessAccount,
    permissions: AllowedPermission,
) -> Result<web::Json<GenericResponse<()>>, GenericError> {
    let leave_type_id = path.into_inner();
    delete_user_leave(&pool, leave_type_id, business_account.id)
        .await
        .map_err(|e| {
            GenericError::DatabaseError(
                "Something went wrong while deleting user leaves".to_string(),
                e,
            )
        })?;
    Ok(web::Json(GenericResponse::success(
        "Sucessfully deleted user leaves",
        (),
    )))
}

#[utoipa::path(
    post,
    description = "API for making a leave request",
    tag = "Leave",
    summary = "Leave Request Creation API",
    path = "/leave/request/create",
    request_body(content = CreateLeaveRequest, description = "Request Body"),
    responses(
        (status=200, description= "project Account created successfully", body= GenericResponse<TupleUnit>),
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
#[tracing::instrument(err, name = "Leave Request Creation API", skip(pool, body), fields())]
pub async fn create_leave_req(
    body: CreateLeaveRequest,
    pool: web::Data<PgPool>,
    user: UserAccount,
    business: BusinessAccount,
    mail_config: web::Data<EmailClientConfig>,
    permissions: AllowedPermission,
    websocket_srv: web::Data<Addr<Server>>,
) -> Result<web::Json<GenericResponse<()>>, GenericError> {
    if body.user_id.is_some()
        && !permissions
            .permission_list
            .contains(&PermissionType::CreateLeaveRequest.to_string())
    {
        return Err(GenericError::InsufficientPrevilegeError(
            "You don't have sufficient previlege to create leave request for other users"
                .to_string(),
        ));
    }
    let user_id = body.user_id.unwrap_or(user.id);
    if !body.send_mail && !user.is_vector_verified(&VectorType::Email) {
        return Err(GenericError::InsufficientPrevilegeError(
            "Please Verify your email, before creating a leave request".to_string(),
        ));
    }
    let setting_keys = vec![
        SettingKey::EmailAppPassword.to_string(),
        SettingKey::LeaveRequestTemplate.to_string(),
    ];
    let (config_res, reciever_account_res, user_leave_res) = join!(
        get_setting_value(&pool, &setting_keys, None, Some(user.id), true),
        get_user(vec![body.to.get()], &pool),
        // get_leave_type(&pool, business.id, Some(vec![body.r#type]), None, None),
        // get_leave_group(&pool, business.id, None, None, Some(Utc::now()), Some(Utc::now())),
        fetch_user_leaves(&pool, business.id, user_id, None, Some(body.user_leave_id)),
    );
    let user_leave_list =
        user_leave_res.map_err(|e| GenericError::DatabaseError(e.to_string(), e))?;
    let user_leave = user_leave_list.first().ok_or_else(|| {
        GenericError::DataNotFound(
            "No Leave is added for the user for given group and type is not set fot user"
                .to_string(),
        )
    })?;
    // .map_err(|e| GenericError::DatabaseError(e.to_string(), e))?;
    let configs = config_res.map_err(|e| GenericError::DatabaseError(e.to_string(), e))?;

    validate_leave_request_creation(&body, user_leave)
        .map_err(|e| GenericError::ValidationError(e.to_string()))?;

    let email_password = configs
        .get_setting(&SettingKey::EmailAppPassword.to_string())
        .ok_or_else(|| {
            GenericError::DataNotFound(format!("Please set the {}", SettingKey::EmailAppPassword))
        })?;

    let personal_email_client = SmtpEmailClient::new_personal(
        &user.email,
        SecretString::from(email_password.as_ref()),
        &mail_config.personal.base_url,
    )
    .unwrap();

    let message_id = if body.send_mail {
        Some(personal_email_client.generate_message_id(&mail_config.personal.message_id_suffix))
    } else {
        None
    };

    let reciever_account = reciever_account_res
        .map_err(|e| GenericError::DatabaseError(e.to_string(), e))?
        .ok_or(GenericError::DataNotFound("User not found.".to_string()))?;

    let mut transaction = pool
        .begin()
        .await
        .context("Failed to acquire a Postgres connection from the pool")?;
    if save_leave_request(
        &mut transaction,
        &body,
        user_leave.id,
        user.id,
        reciever_account.id,
        message_id.as_deref(),
    )
    .await
    .map_err(|e| GenericError::DatabaseError(e.to_string(), e))?
    {
        if body.send_mail {
            let html_template: String = configs
                .get_setting(&SettingKey::LeaveRequestTemplate.to_string())
                .ok_or_else(|| {
                    GenericError::DataNotFound(format!(
                        "Please set the {}",
                        SettingKey::LeaveRequestTemplate
                    ))
                })?;
            let receiver = to_title_case(&reciever_account.display_name);
            let sender = to_title_case(&user.display_name);
            let reason = body.reason.unwrap_or("NA".to_string());
            let context_data = LeaveRequestEmailContext::new(
                &sender,
                body.leave_data.iter().map(|a| a.date.to_string()).collect(),
                &reason,
                &receiver,
                &user_leave.leave_type.label,
            );
            let context =
                TeraContext::from_serialize(&context_data).map_err(|e: tera::Error| {
                    tracing::error!("{}", e);
                    GenericError::UnexpectedCustomError(
                        "Something went wrong while rendering the email html data".to_string(),
                    )
                })?;
            let rendered_string = Tera::one_off(&html_template, &context, true).map_err(|e| {
                tracing::error!("Error while rendering html {} error: {}", html_template, e);
                GenericError::UnexpectedCustomError(
                    "Something went wrong while rendering the email html data".to_string(),
                )
            })?;
            personal_email_client
                .send_html_email(
                    &body.to,
                    &body.cc,
                    &format!("Request for {} leave", &user_leave.leave_type.label),
                    rendered_string,
                    message_id,
                    None,
                )
                .await
                .map_err(|e| GenericError::UnexpectedCustomError(e.to_string()))?;
        }

        let _ = send_notification(
            &pool,
            &websocket_srv,
            WebSocketActionType::LeaveRequest,
            ProcessType::Deferred,
            Some(reciever_account.id),
            format!("Leave Request send by {}", user.display_name),
            Some(business.id),
        )
        .await;
        // .map_err(|e| GenericError::UnexpectedCustomError(e.to_string()))?;
    }
    transaction
        .commit()
        .await
        .context("Failed to commit SQL transaction to store a new user account.")?;

    Ok(web::Json(GenericResponse::success(
        "Sucessfully created leave request",
        (),
    )))
}

#[utoipa::path(
    delete,
    description = "API for making a deleting leave request",
    tag = "Leave",
    summary = "Leave Request Deletion API",
    path = "/leave/request/delete",
    responses(
        (status=200, description= "project Account created successfully", body= GenericResponse<TupleUnit>),
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
        ("id" = String, Path, description = "Leave ID"),
      )
)]
#[tracing::instrument(err, name = "Leave Request Deletion Request", skip(pool), fields())]
pub async fn leave_request_deletion_req(
    path: web::Path<Uuid>,
    pool: web::Data<PgPool>,
    user: UserAccount,
    mail_config: web::Data<EmailClientConfig>,
    // permissions: AllowedPermission,
) -> Result<web::Json<GenericResponse<()>>, GenericError> {
    let leave_id = path.into_inner();

    let filter_query = FetchLeaveQuery::builder().with_leave_id(Some(leave_id));
    let leave = get_leaves(&pool, &filter_query)
        .await
        .map_err(|e| {
            GenericError::DatabaseError(
                "Something went wrong while fetching leave data".to_string(),
                e,
            )
        })?
        .into_iter()
        .next()
        .ok_or_else(|| GenericError::DataNotFound("Invalid Leave ID".to_string()))?;

    if leave.user_id != user.id {
        return Err(GenericError::InsufficientPrevilegeError(
            "You don't have previlege to delete other user's leaves".to_string(),
        ));
    };
    if leave.status == LeaveStatus::Approved {
        return Err(GenericError::ValidationError(
            "The leave is already approved, pls reject the leave request before deletion"
                .to_string(),
        ));
    }
    delete_leave(&pool, leave_id, user.id).await.map_err(|_| {
        GenericError::UnexpectedCustomError("something went wrong while deleting leave".to_string())
    })?;
    Ok(web::Json(GenericResponse::success(
        "Sucessfully deleted leave request",
        (),
    )))
}

#[utoipa::path(
    patch,
    description = "API for making a updating leave status",
    tag = "Leave",
    summary = "Leave Request Status Updation API",
    path = "/leave/request/status/update",
    request_body(content = UpdateLeaveStatusRequest, description = "Request Body"),
    responses(
        (status=200, description= "project Account created successfully", body= GenericResponse<TupleUnit>),
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
    name = "Leave Request Status Updation API",
    skip(pool, body, producer_client),
    fields()
)]
#[allow(clippy::too_many_arguments)]
pub async fn update_leave_status_req(
    body: UpdateLeaveStatusRequest,
    pool: web::Data<PgPool>,
    user: UserAccount,
    mail_config: web::Data<EmailClientConfig>,
    permissions: AllowedPermission,
    websocket_srv: web::Data<Addr<Server>>,
    producer_client: web::Data<PulsarClient>,
    business: BusinessAccount,
) -> Result<web::Json<GenericResponse<()>>, GenericError> {
    let filter_query = FetchLeaveQuery::builder().with_leave_id(Some(body.id));
    let leave: LeaveRequestData = get_leaves(&pool, &filter_query)
        .await
        .map_err(|e| {
            GenericError::DatabaseError(
                "Something went wrong while fetching leave data".to_string(),
                e,
            )
        })?
        .into_iter()
        .next()
        .ok_or_else(|| {
            GenericError::DataNotFound("Provided Leave Request not found in database".to_string())
        })?;

    let user_leave = fetch_user_leaves(
        &pool,
        business.id,
        leave.user_id,
        None,
        Some(leave.user_leave_id),
    )
    .await
    .map_err(|e| {
        GenericError::DatabaseError(
            "Something went wrong while fetching  user leave data".to_string(),
            e,
        )
    })?
    .into_iter()
    .next()
    .ok_or_else(|| {
        GenericError::DataNotFound("Provided Leave Request not found in database".to_string())
    })?;
    validate_leave_status_update(
        &body.status,
        &leave.status,
        &permissions,
        &user_leave,
        &leave.period,
    )?;
    let mut transaction = pool
        .begin()
        .await
        .context("Failed to acquire a Postgres connection from the pool")?;

    update_leave_request_status(&mut transaction, body.id, &body.status, user.id)
        .await
        .map_err(|e| {
            GenericError::DatabaseError(
                "Something went wrong while updating leave request".to_string(),
                e,
            )
        })?;
    if body.status == LeaveStatus::Approved || body.status == LeaveStatus::Cancelled {
        let adjustment = match leave.status {
            LeaveStatus::Approved => leave.period.value.clone(),
            LeaveStatus::Cancelled => -&leave.period.value,
            _ => BigDecimal::default(),
        };

        update_user_leave_count(&mut transaction, leave.user_leave_id, &adjustment, user.id)
            .await
            .map_err(|e| {
                GenericError::DatabaseError(
                    "Something went wrong while updating leave count".to_string(),
                    e,
                )
            })?;
    }

    let setting_value_list = vec![
        SettingKey::LeaveRequestStatusUpdateTemplate.to_string(),
        SettingKey::EmailAppPassword.to_string(),
    ];
    let reciever_id = leave.user_id.to_string();
    let (config_res, reciever_account_res) = join!(
        get_setting_value(&pool, &setting_value_list, None, Some(user.id), true),
        get_user(vec![&reciever_id], &pool),
    );
    let configs = config_res.map_err(|e| GenericError::DatabaseError(e.to_string(), e))?;
    let reciever_account = reciever_account_res
        .map_err(|e| GenericError::DatabaseError(e.to_string(), e))?
        .ok_or(GenericError::DataNotFound("User not found.".to_string()))?;

    let receiver = to_title_case(&reciever_account.display_name);
    let sender = to_title_case(&user.display_name);
    if body.status == LeaveStatus::Approved {
        let msg = SchedulerMessageData {
            partition_key: None,
            date: leave.date,
        };
        let mut producer = producer_client
            .get_producer(producer_client.get_product_topic(PulsarTopic::Scheduler))
            .await;

        let msg = producer
            .create_message()
            .with_content(msg)
            .deliver_at(leave.date.into())
            .map_err(|e| GenericError::UnexpectedError(e.into()))?;
        msg.send_non_blocking()
            .await
            .map_err(|e| GenericError::UnexpectedError(e.into()))?;
    }
    //     if !user.is_vector_verified(&VectorType::Email) {
    //     return Err(GenericError::InsufficientPrevilegeError(
    //         "Please Verify your email, before updating leave requst status".to_string(),
    //     ));
    // }
    if leave.email_message_id.is_some() && user.is_vector_verified(&VectorType::Email) {
        let html_template: String = configs
            .get_setting(&SettingKey::LeaveRequestStatusUpdateTemplate.to_string())
            .ok_or_else(|| {
                GenericError::DataNotFound(format!(
                    "Please set the {}",
                    SettingKey::LeaveRequestStatusUpdateTemplate
                ))
            })?;
        let context_data =
            LeaveRequestStatusEmailContext::new(&sender, &receiver, &body.status, &leave.date);
        let context = TeraContext::from_serialize(&context_data).map_err(|e: tera::Error| {
            tracing::error!("{}", e);
            GenericError::UnexpectedCustomError(
                "Something went wrong while rendering the email html data".to_string(),
            )
        })?;
        let rendered_string = Tera::one_off(&html_template, &context, true).map_err(|e| {
            tracing::error!("Error while rendering html {} error: {}", html_template, e);
            GenericError::UnexpectedCustomError(
                "Something went wrong while rendering the email html data".to_string(),
            )
        })?;
        let email_password = configs
            .get_setting(&SettingKey::EmailAppPassword.to_string())
            .ok_or_else(|| {
                GenericError::DataNotFound(format!(
                    "Please set the {}",
                    SettingKey::EmailAppPassword
                ))
            })?;

        let personal_email_client = SmtpEmailClient::new_personal(
            &user.email,
            SecretString::from(email_password.as_ref()),
            &mail_config.personal.base_url,
        )
        .unwrap();
        personal_email_client
            .send_html_email(
                &reciever_account.email,
                &leave.cc,
                &format!("Request for {} leave", leave.leave_type),
                rendered_string,
                leave.email_message_id.clone(),
                leave.email_message_id,
            )
            .await
            .map_err(|e| GenericError::UnexpectedCustomError(e.to_string()))?;
    }

    let _ = send_notification(
        &pool,
        &websocket_srv,
        WebSocketActionType::LeaveRequestStatusUpdation,
        ProcessType::Deferred,
        Some(reciever_account.id),
        format!("Leave Request send by {}", user.display_name),
        Some(business.id),
    )
    .await;
    // .map_err(|e| GenericError::UnexpectedCustomError(e.to_string()))?;

    transaction
        .commit()
        .await
        .context("Failed to commit SQL transaction to store a new user account.")?;

    Ok(web::Json(GenericResponse::success(
        "Sucessfully updated leave request status",
        (),
    )))
}

#[utoipa::path(
    post,
    description = "API for creating leave period",
    tag = "Leave",
    summary = "Leave Period Creation API",
    path = "/leave/period/create",
    request_body(content = LeavePeriodCreationRequest, description = "Request Body"),
    responses(
        (status=200, description= "project Account created successfully", body= GenericResponse<TupleUnit>),
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
        ("x-business-id" = String, Header, description = "id of business_account"),
      )
)]
#[tracing::instrument(err, name = "Leave group create request", skip(pool), fields())]
pub async fn leave_period_create_req(
    pool: web::Data<PgPool>,
    req: LeavePeriodCreationRequest,
    user: UserAccount,
    business_account: BusinessAccount,
) -> Result<web::Json<GenericResponse<()>>, GenericError> {
    let label_list = req
        .data
        .iter()
        .filter(|a| a.id.is_none())
        .map(|a| a.label.as_str())
        .collect();
    let leave_type_list =
        get_leave_period(&pool, business_account.id, None, Some(label_list), None)
            .await
            .map_err(|e| {
                GenericError::DatabaseError(
                    "Something went wrong while fetching leave period".to_string(),
                    e,
                )
            })?;

    if !leave_type_list.is_empty() {
        let existing_labels = leave_type_list
            .iter()
            .map(|p| p.label.as_str())
            .collect::<Vec<&str>>()
            .join(", ");

        return Err(GenericError::ValidationError(format!(
            "Leave periods with label(s) already exist: {}",
            existing_labels
        )));
    }

    save_leave_period(&pool, &req.data, user.id, business_account.id)
        .await
        .map_err(|e| {
            GenericError::DatabaseError(
                "Something went wrong while saving leave period".to_string(),
                e,
            )
        })?;
    Ok(web::Json(GenericResponse::success(
        "Sucessfully saved leave period",
        (),
    )))
}

#[utoipa::path(
    post,
    description = "API for listing leave period",
    tag = "Leave",
    summary = "Leave Period List API",
    path = "/leave/period/list",
    request_body(content = LeaveTypeFetchRequest, description = "Request Body"),
    responses(
        (status=200, description= "project Account created successfully", body= GenericResponse<Vec<LeavePeriodData>>),
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
        ("x-business-id" = String, Header, description = "id of business_account"),
        // ("id" = String, Path, description = "Leave ID"),
      )
)]
#[tracing::instrument(err, name = "Leave type list request", skip(pool), fields())]
pub async fn leave_period_list_req(
    pool: web::Data<PgPool>,
    req: LeavePeriodFetchRequest,
    user: UserAccount,
    business_account: BusinessAccount,
) -> Result<web::Json<GenericResponse<Vec<LeavePeriodData>>>, GenericError> {
    let leave_type_list = get_leave_period(&pool, business_account.id, None, None, req.query)
        .await
        .map_err(|e| {
            GenericError::DatabaseError(
                "Something went wrong while fetching leave period".to_string(),
                e,
            )
        })?;

    Ok(web::Json(GenericResponse::success(
        "Sucessfully fetched leave period",
        leave_type_list,
    )))
}

#[utoipa::path(
    delete,
    description = "API for deleting leave period",
    tag = "Leave",
    summary = "Leave Group Delete API",
    path = "/leave/period/delete",
    // request_body(content = FetchLeaveRequest, description = "Request Body"),
    responses(
        (status=200, description= "project Account created successfully", body= GenericResponse<TupleUnit>),
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
        ("x-business-id" = String, Header, description = "id of business_account"),
        ("id" = String, Path, description = "Leave Period ID"),
      )
)]
#[tracing::instrument(err, name = "Leave period delete request", skip(pool), fields())]
pub async fn leave_period_delete_req(
    path: web::Path<Uuid>,
    pool: web::Data<PgPool>,
    user: UserAccount,
    business_account: BusinessAccount,
) -> Result<web::Json<GenericResponse<()>>, GenericError> {
    let leave_period_id = path.into_inner();
    let leave_group_list = get_leave_period(
        &pool,
        business_account.id,
        Some(&vec![leave_period_id]),
        None,
        None,
    )
    .await
    .map_err(|e| {
        GenericError::DatabaseError(
            "Something went wrong while fetching leave period".to_string(),
            e,
        )
    })?;
    leave_group_list
        .first()
        .ok_or_else(|| GenericError::DataNotFound("Invalid Leave period id".to_string()))?;

    delete_leave_period(&pool, leave_period_id)
        .await
        .map_err(|e| {
            GenericError::DatabaseError(
                "Something went wrong while deleting leave period".to_string(),
                e,
            )
        })?;

    Ok(web::Json(GenericResponse::success(
        "Sucessfully deleted leave period",
        (),
    )))
}
