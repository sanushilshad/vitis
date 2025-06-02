use actix_web::web;
use secrecy::SecretString;
use sqlx::PgPool;
use utoipa::TupleUnit;

use crate::{
    email_client::{GenericEmailService, SmtpEmailClient},
    errors::GenericError,
    routes::user::schemas::UserAccount,
    schemas::GenericResponse,
};

use super::schemas::CreateLeaveRequest;

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
        ("x-project-id" = String, Header, description = "id of project_account"),
        ("x-request-id" = String, Header, description = "Request id"),
        ("x-device-id" = String, Header, description = "Device id"),
      )
)]
#[tracing::instrument(err, name = "Leave Request Creation API", skip(_pool, _body), fields())]
pub async fn create_leave_req(
    _body: CreateLeaveRequest,
    _pool: web::Data<PgPool>,
    user: UserAccount,
    // email_client: web::Data<SmtpEmailClient>,
) -> Result<web::Json<GenericResponse<()>>, GenericError> {
    let personal_email_client = SmtpEmailClient::new_personal(
        user.email,
        SecretString::new("uazy ptlw drfp jrjr".into()),
        "smtp.gmail.com",
    )
    .unwrap();
    let responsed = personal_email_client
        .send_text_email(
            "sanu.shilshad@acelrtech.com",
            "SANU",
            "apple".to_owned(),
            None,
            None,
        )
        .await;
    Ok(web::Json(GenericResponse::success(
        "Sucessfully fetched user config/s",
        (),
    )))
}
