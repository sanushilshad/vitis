use actix_web::web;
use chrono::{DateTime, Utc};
use chrono_tz::Tz;
use secrecy::SecretString;
use sqlx::PgPool;
use tera::{Context, Tera};
use tokio::join;
use utoipa::TupleUnit;
use uuid::Uuid;

use crate::{
    configuration::EmailClientConfig,
    email_client::{GenericEmailService, SmtpEmailClient},
    errors::GenericError,
    routes::{
        project::schemas::{AllowedPermission, PermissionType},
        setting::{
            schemas::{SettingKey, SettingsExt},
            utils::get_setting_value,
        },
        user::{schemas::UserAccount, utils::get_user},
    },
    schemas::GenericResponse,
    utils::to_title_case,
};

use super::{
    schemas::{
        CreateLeaveRequest, FetchLeaveQuery, FetchLeaveRequest, FetchLeaveType, LeaveData,
        LeaveRequestEmailContext, LeaveRequestStatusEmailContext, UpdateLeaveStatusRequest,
    },
    utils::{
        delete_leave, get_leaves, save_leave_request, update_leave_status, validate_leave_request,
        validate_leave_status_update,
    },
};

#[utoipa::path(
    post,
    description = "API for making a leave request",
    tag = "Leave",
    summary = "Leave Request Creation API",
    path = "/leave/create",
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
    mail_config: web::Data<EmailClientConfig>,
    permissions: AllowedPermission,
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
    let setting_keys = vec![
        SettingKey::EmailAppPassword.to_string(),
        body.r#type.get_setting_key().to_string(),
        SettingKey::FinancialYearStart.to_string(),
        SettingKey::LeaveRequestTemplate.to_string(),
    ];
    let (config_res, reciever_account_res) = join!(
        get_setting_value(&pool, &setting_keys, None, user.id,),
        get_user(vec![body.to.get()], &pool),
    );
    // .map_err(|e| GenericError::DatabaseError(e.to_string(), e))?;
    let configs = config_res.map_err(|e| GenericError::DatabaseError(e.to_string(), e))?;

    let email_password = configs
        .get_setting(&SettingKey::EmailAppPassword.to_string())
        .ok_or_else(|| {
            GenericError::DataNotFound(format!("Please set the {}", SettingKey::EmailAppPassword))
        })?;
    let allowed_leave_count = configs
        .get_setting(&body.r#type.get_setting_key().to_string())
        .ok_or_else(|| {
            GenericError::DataNotFound(format!(
                "Please set the {}",
                &body.r#type.get_setting_key().to_string()
            ))
        })?;
    let financial_year_start = configs
        .get_setting(&SettingKey::FinancialYearStart.to_string())
        .ok_or_else(|| {
            GenericError::DataNotFound(format!("Please set the {}", SettingKey::FinancialYearStart))
        })?;
    let html_template: String = configs
        .get_setting(&SettingKey::LeaveRequestTemplate.to_string())
        .ok_or_else(|| {
            GenericError::DataNotFound(format!(
                "Please set the {}",
                SettingKey::LeaveRequestTemplate
            ))
        })?;
    validate_leave_request(
        &pool,
        DateTime::parse_from_str(&financial_year_start, "%Y-%m-%d %H:%M:%S%.f%:z")
            .unwrap()
            .with_timezone(&Utc),
        &body,
        user.id,
        allowed_leave_count.parse::<i32>().unwrap(),
    )
    .await
    .map_err(|e| GenericError::ValidationError(e.to_string()))?;
    let personal_email_client = SmtpEmailClient::new_personal(
        &user.email,
        SecretString::from(email_password.as_ref()),
        &mail_config.personal.base_url,
    )
    .unwrap();
    let message_id =
        personal_email_client.generate_message_id(&mail_config.personal.message_id_suffix);
    let reciever_account =
        reciever_account_res.map_err(|e| GenericError::DatabaseError(e.to_string(), e))?;
    if save_leave_request(&pool, &body, user.id, reciever_account.id, &message_id)
        .await
        .map_err(|e| GenericError::DatabaseError(e.to_string(), e))?
    {
        let name = to_title_case(&reciever_account.display_name);
        let sender = to_title_case(&user.display_name);
        let reason = body.reason.unwrap_or("NA".to_string());
        let context_data = LeaveRequestEmailContext::new(
            &name,
            body.leave_data.iter().map(|a| a.date.to_string()).collect(),
            &reason,
            &sender,
            &body.r#type,
        );
        let context = Context::from_serialize(&context_data).map_err(|e: tera::Error| {
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
                &format!("Request for {} leave", body.r#type),
                rendered_string,
                Some(message_id),
                None,
            )
            .await
            .map_err(|e| GenericError::UnexpectedCustomError(e.to_string()))?;
    }

    Ok(web::Json(GenericResponse::success(
        "Sucessfully created leave request",
        (),
    )))
}

#[utoipa::path(
    patch,
    description = "API for making a updating leave status",
    tag = "Leave",
    summary = "Leave Request Status Updation API",
    path = "/leave/status/update",
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
    skip(pool, body),
    fields()
)]
pub async fn update_leave_status_req(
    body: UpdateLeaveStatusRequest,
    pool: web::Data<PgPool>,
    user: UserAccount,
    mail_config: web::Data<EmailClientConfig>,
    permissions: AllowedPermission,
) -> Result<web::Json<GenericResponse<()>>, GenericError> {
    let filter_query = FetchLeaveQuery::builder().with_leave_id(Some(body.id));
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
        .ok_or_else(|| {
            GenericError::DataNotFound("Provided Leave Request not found in database".to_string())
        })?;
    validate_leave_status_update(&body.status, &leave.status, &permissions)?;

    update_leave_status(&pool, body.id, &body.status, user.id)
        .await
        .map_err(|e| GenericError::DatabaseError("Leave Data not found".to_string(), e))?;

    if user.id != leave.sender_id {
        let setting_value_list = vec![
            SettingKey::LeaveStatusUpdateTemplate.to_string(),
            SettingKey::EmailAppPassword.to_string(),
        ];
        let reciever_id = leave.sender_id.to_string();
        let (config_res, reciever_account_res) = join!(
            get_setting_value(&pool, &setting_value_list, None, user.id,),
            get_user(vec![&reciever_id], &pool),
        );
        let configs = config_res.map_err(|e| GenericError::DatabaseError(e.to_string(), e))?;
        let reciever_account =
            reciever_account_res.map_err(|e| GenericError::DatabaseError(e.to_string(), e))?;
        let html_template: String = configs
            .get_setting(&SettingKey::LeaveStatusUpdateTemplate.to_string())
            .ok_or_else(|| {
                GenericError::DataNotFound(format!(
                    "Please set the {}",
                    SettingKey::LeaveStatusUpdateTemplate
                ))
            })?;
        let name = to_title_case(&reciever_account.display_name);
        let sender = to_title_case(&user.display_name);
        let context_data =
            LeaveRequestStatusEmailContext::new(&name, &sender, &body.status, &leave.date);
        let context = Context::from_serialize(&context_data).map_err(|e: tera::Error| {
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
                &format!("Request for {} leave", leave.r#type),
                rendered_string,
                leave.email_message_id.to_owned(),
                leave.email_message_id.to_owned(),
            )
            .await
            .map_err(|e| GenericError::UnexpectedCustomError(e.to_string()))?;
    }

    Ok(web::Json(GenericResponse::success(
        "Sucessfully updated leave request status",
        (),
    )))
}

#[utoipa::path(
    delete,
    description = "API for making a deleting leave request",
    tag = "Leave",
    summary = "Leave Request Deletion API",
    path = "/leave/delete",
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
    permissions: AllowedPermission,
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

    if leave.sender_id != user.id {
        return Err(GenericError::InsufficientPrevilegeError(
            "You don't have previlege to delete other user's leaves".to_string(),
        ));
    };
    delete_leave(&pool, leave_id, user.id).await.map_err(|_| {
        GenericError::UnexpectedCustomError(
            "You don't have previlege to delete other user's leaves".to_string(),
        )
    })?;
    Ok(web::Json(GenericResponse::success(
        "Sucessfully deleted leave request",
        (),
    )))
}

#[utoipa::path(
    post,
    description = "API for making fetching leave request",
    tag = "Leave",
    summary = "Leave Request Fetch API",
    path = "/leave/fetch",
    request_body(content = FetchLeaveRequest, description = "Request Body"),
    responses(
        (status=200, description= "project Account created successfully", body= GenericResponse<Vec<LeaveData>>),
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
#[tracing::instrument(err, name = "Leave Request fetch request", skip(pool), fields())]
pub async fn leave_request_fetch_req(
    pool: web::Data<PgPool>,
    req: FetchLeaveRequest,
    user: UserAccount,
    mail_config: web::Data<EmailClientConfig>,
) -> Result<web::Json<GenericResponse<Vec<LeaveData>>>, GenericError> {
    let setting_key_list = vec![SettingKey::TimeZone.to_string()];
    let setting_list = get_setting_value(&pool, &setting_key_list, None, user.id)
        .await
        .map_err(|e| GenericError::DatabaseError(e.to_string(), e))?;
    let timezone = setting_list
        .get_setting(&SettingKey::TimeZone.to_string())
        .ok_or_else(|| {
            GenericError::DataNotFound(format!("Please set the {}", SettingKey::EmailAppPassword))
        })?;
    let sender_id = if req.action_type == FetchLeaveType::Sender {
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
