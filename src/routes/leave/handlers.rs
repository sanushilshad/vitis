use actix_web::web;
use chrono::{DateTime, Utc};
use secrecy::SecretString;
use sqlx::PgPool;
use tera::{Context, Tera};
use tokio::join;
use utoipa::TupleUnit;

use crate::{
    configuration::EmailClientConfig,
    email_client::{GenericEmailService, SmtpEmailClient},
    errors::GenericError,
    routes::{
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
    schemas::{CreateLeaveRequest, LeaveRequestEmailContext},
    utils::{save_leave_request, validate_leave_request},
};

#[utoipa::path(
    post,
    description = "API for making a leave request",
    tag = "Leave",
    summary = "Leave Request Creation",
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
) -> Result<web::Json<GenericResponse<()>>, GenericError> {
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
            GenericError::DataNotFound(format!("Please set the {}", SettingKey::FinancialYearStart))
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

    if save_leave_request(&pool, &body, user.id, &message_id)
        .await
        .map_err(|e| GenericError::DatabaseError(e.to_string(), e))?
    {
        let reciever_account =
            reciever_account_res.map_err(|e| GenericError::DatabaseError(e.to_string(), e))?;
        let name = to_title_case(&reciever_account.display_name);
        let sender = to_title_case(&user.display_name);
        let reason = body.reason.unwrap_or("NA".to_string());
        let context_data = LeaveRequestEmailContext::new(
            &name,
            body.leave_data.iter().map(|a| a.date.to_string()).collect(),
            &reason,
            &sender,
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
