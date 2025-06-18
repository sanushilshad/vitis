use actix_web::web;
use rand::Rng;
use sqlx::PgPool;
use tokio::join;
use utoipa::TupleUnit;

use crate::{
    configuration::{SecretConfig, UserConfig},
    email_client::SmtpEmailClient,
    errors::GenericError,
    routes::setting::{schemas::SettingKey, utils::get_setting_value},
    schemas::{DeleteType, GenericResponse, RequestMetaData, Status},
    whatsapp_client::{TemplateType, WhatsAppClient},
};

use super::{
    errors::UserRegistrationError,
    schemas::{
        AuthData, AuthenticateRequest, AuthenticationScope, CreateUserAccount, EditUserAccount,
        ListUserAccountRequest, MinimalUserAccount, PasswordResetReq, SendOTPRequest, UserAccount,
        UserRoleType,
    },
    utils::{
        fetch_user, get_auth_data, get_minimal_user_list, get_stored_credentials, get_user,
        hard_delete_user_account, reactivate_user_account, register_user, reset_password,
        send_email_otp, soft_delete_user_account, update_otp, update_user,
        validate_user_credentials, verify_vector_if_needed,
    },
};

#[utoipa::path(
    post,
    path = "/user/register",
    tag = "User Account",
    description = "API for creating user accounts. The username, email and mobile no should be unique",
    summary = "User Account Registration API",
    request_body(content = CreateUserAccount, description = "Request Body"),
    responses(
        (status=200, description= "Account created successfully", body= GenericResponse<TupleUnit> ),
        (status=400, description= "Invalid Request body", body= GenericResponse<TupleUnit>),
        (status=401, description= "Invalid Token", body= GenericResponse<TupleUnit>),
	    (status=403, description= "Insufficient Previlege", body= GenericResponse<TupleUnit>),
	    (status=410, description= "Data not found", body= GenericResponse<TupleUnit>),
        (status=500, description= "Internal Server Error", body= GenericResponse<TupleUnit>)
    ),
    params(
        ("x-request-id" = String, Header, description = "Request id"),
        ("x-device-id" = String, Header, description = "Device id"),
    )
)]
#[tracing::instrument(
    err,
    name = "User Account Registration API",
    skip(pool, body),
    fields()
)]
pub async fn register_user_account_req(
    body: CreateUserAccount,
    pool: web::Data<PgPool>,
    meta_data: RequestMetaData,
    user_settings: web::Data<UserConfig>,
) -> Result<web::Json<GenericResponse<()>>, GenericError> {
    let admin_role = [UserRoleType::Admin, UserRoleType::Superadmin];
    if admin_role.contains(&body.user_type) && !user_settings.admin_list.contains(&body.mobile_no) {
        return Err(UserRegistrationError::InsufficientPrevilegeError(
            "Insufficient previlege to register Admin/Superadmin".to_string(),
        )
        .into());
    } else {
        match register_user(&pool, &body).await {
            Ok(uuid) => {
                tracing::Span::current().record("user_id", tracing::field::display(uuid));
                Ok(web::Json(GenericResponse::success(
                    "Sucessfully Registered User",
                    (),
                )))
            }
            Err(e) => {
                tracing::error!("Failed to register user: {:?}", e);
                return Err(e.into());
            }
        }
    }
}

#[utoipa::path(
    post,
    path = "/user/authenticate",
    tag = "User Account",
    description = "API for authentication. currently only supports password authentication",
    summary = "Authentication API",
    request_body(content = AuthenticateRequest, description = "Request Body"),
    responses(
        (status=200, description= "Authenticate User", body= GenericResponse<AuthData>),
        (status=400, description= "Invalid Request body", body= GenericResponse<TupleUnit>),
        (status=401, description= "Invalid Token", body= GenericResponse<TupleUnit>),
	    (status=403, description= "Insufficient Previlege", body= GenericResponse<TupleUnit>),
	    (status=410, description= "Data not found", body= GenericResponse<TupleUnit>),
        (status=500, description= "Internal Server Error", body= GenericResponse<TupleUnit>)
    ),
    params(
        ("x-request-id" = String, Header, description = "Request id"),
        ("x-device-id" = String, Header, description = "Device id"),
    )
)]
#[tracing::instrument(err, name = "Authenticate User", skip(pool, body), fields())]
pub async fn authenticate_req(
    body: AuthenticateRequest,
    pool: web::Data<PgPool>,
    secret_obj: web::Data<SecretConfig>,
) -> Result<web::Json<GenericResponse<AuthData>>, GenericError> {
    tracing::Span::current().record("identifier", tracing::field::display(&body.identifier));

    let user_id = validate_user_credentials(&body, &pool)
        .await?
        .ok_or_else(|| {
            GenericError::UnexpectedCustomError("Unknown user credential".to_string())
        })?;
    tracing::Span::current().record("user_id", tracing::field::display(&user_id));

    let user_model = fetch_user(vec![&user_id.to_string()], &pool)
        .await
        .map_err(|e| {
            GenericError::DatabaseError("Something went wrong while fetching user".to_string(), e)
        })?
        .ok_or_else(|| GenericError::DataNotFound("User not found".to_string()))?;

    let user_account = user_model.into_schema();

    verify_vector_if_needed(&pool, &user_account, &body.scope)
        .await
        .map_err(|e| {
            GenericError::DatabaseError(
                "Something went wrong while status of vector".to_string(),
                e,
            )
        })?;

    let auth_obj = get_auth_data(user_account, &secret_obj.jwt)?;
    Ok(web::Json(GenericResponse::success(
        "Successfully Authenticated User",
        auth_obj,
    )))
}

#[utoipa::path(
    post,
    path = "/user/fetch",
    tag = "User Account",
    description = "API for fetching user account detail.",
    summary = "User Account Fetch API",
    responses(
        (status=200, description=    "Sucessfully fetched user data.", body= GenericResponse<UserAccount>),
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
#[tracing::instrument(err, name = "fetch user detail", skip(), fields())]
pub async fn fetch_user_req(
    user_account: UserAccount,
) -> Result<web::Json<GenericResponse<UserAccount>>, GenericError> {
    Ok(web::Json(GenericResponse::success(
        "Sucessfully fetched user data.",
        user_account,
    )))
}

#[utoipa::path(
    post,
    path = "/user/send/otp",
    tag = "User Account",
    description = "API for sending OTP",
    summary = "Send OTP API",
    request_body(content = SendOTPRequest, description = "Request Body"),
    responses(
        (status=200, description= "Sucessfully send data.", body= GenericResponse<TupleUnit>),
        (status=400, description= "Invalid Request body", body= GenericResponse<TupleUnit>),
        (status=401, description= "Invalid Token", body= GenericResponse<TupleUnit>),
	    (status=403, description= "Insufficient Previlege", body= GenericResponse<TupleUnit>),
	    (status=410, description= "Data not found", body= GenericResponse<TupleUnit>),
        (status=500, description= "Internal Server Error", body= GenericResponse<TupleUnit>)
    ),
    params(
        ("x-request-id" = String, Header, description = "Request id"),
        ("x-device-id" = String, Header, description = "Device id"),
    )
)]
#[tracing::instrument(err, name = "send OTP", skip(pool, email_client), fields())]
pub async fn send_otp_req(
    req: SendOTPRequest,
    pool: web::Data<PgPool>,
    secret_obj: web::Data<SecretConfig>,
    email_client: web::Data<SmtpEmailClient>,
    whatsapp_client: web::Data<WhatsAppClient>,
) -> Result<web::Json<GenericResponse<()>>, GenericError> {
    let user_task_1 = get_user(vec![&req.identifier], &pool);
    let credential_task_2 = get_stored_credentials(&req.identifier, &req.scope, &pool);
    let (user_res, config_res) = join!(user_task_1, credential_task_2);
    let credential = config_res
        .map_err(|e| {
            GenericError::DatabaseError(
                "Something went wrong while fetching auth details".to_string(),
                e,
            )
        })?
        .ok_or_else(|| GenericError::UnexpectedCustomError("User not found".to_string()))?;
    if credential.is_active == Status::Inactive {
        return Err(GenericError::UnexpectedCustomError(
            "Auth Mechanism is disabled".to_string(),
        ));
    }
    let user = user_res
        .map_err(GenericError::UnexpectedError)?
        .ok_or(GenericError::DataNotFound("User not found".to_string()))?;
    let otp = rand::thread_rng().gen_range(1000..=9999).to_string();
    if req.scope == AuthenticationScope::Email {
        let otp_clone = otp.clone();
        let setting_keys = vec![SettingKey::EmailOTPTemplate.to_string()];
        let configs = get_setting_value(&pool, &setting_keys, None, None, false)
            .await
            .map_err(|e| GenericError::DatabaseError(e.to_string(), e))?;
        tokio::spawn(
            async move { send_email_otp(email_client, &user, &otp_clone, &configs).await },
        );
    } else if req.scope == AuthenticationScope::Otp {
        let otp_clone = otp.clone();
        tokio::spawn(async move {
            whatsapp_client
                .send_text(
                    TemplateType::Authentication,
                    &user.mobile_no,
                    vec![&otp_clone],
                    true,
                )
                .await
        });
    }

    update_otp(&pool, &otp.clone(), secret_obj.otp.expiry, credential)
        .await
        .map_err(|_| {
            GenericError::UnexpectedCustomError(
                "Something went wrong while saving OTP data".to_string(),
            )
        })?;

    Ok(web::Json(GenericResponse::success(
        "Sucessfully send data.",
        (),
    )))
}

#[utoipa::path(
    delete,
    path = "/user/delete/{delete_type}",
    tag = "User Account",
    description = "API for delete User Account",
    summary = "Delete User Account API",
    request_body(content = SendOTPRequest, description = "Request Body"),
    responses(
        (status=200, description= "Successfully deleted user.", body= GenericResponse<TupleUnit>),
        (status=400, description= "Invalid Request body", body= GenericResponse<TupleUnit>),
        (status=401, description= "Invalid Token", body= GenericResponse<TupleUnit>),
	    (status=403, description= "Insufficient Previlege", body= GenericResponse<TupleUnit>),
	    (status=410, description= "Data not found", body= GenericResponse<TupleUnit>),
        (status=500, description= "Internal Server Error", body= GenericResponse<TupleUnit>)
    ),
    params(
        ("delete_type" = DeleteType, Path, description = "Delete type (soft or hard)"),
        ("x-request-id" = String, Header, description = "Request id"),
        ("x-device-id" = String, Header, description = "Device id"),
        ("Authorization" = String, Header, description = "JWT token"),
    )
)]
#[tracing::instrument(err, name = "Delete User Account", skip(pool), fields())]
pub async fn delete_user(
    path: web::Path<DeleteType>,
    pool: web::Data<PgPool>,
    user_account: UserAccount,
) -> Result<web::Json<GenericResponse<()>>, GenericError> {
    let delete_type = path.into_inner();
    let identifier = &user_account.id.to_string();

    match delete_type {
        DeleteType::Soft => {
            soft_delete_user_account(
                pool.get_ref(),
                &user_account.id.to_string(),
                user_account.id,
            )
            .await
            .map_err(|e| {
                tracing::error!("Soft delete failed: {:?}", e);
                GenericError::DatabaseError("Failed to soft delete user".to_string(), e)
            })?;
        }
        DeleteType::Hard => {
            hard_delete_user_account(pool.get_ref(), identifier)
                .await
                .map_err(|e| {
                    tracing::error!("Hard delete failed: {:?}", e);
                    GenericError::DatabaseError("Failed to hard delete user".to_string(), e)
                })?;
        }
    }

    Ok(web::Json(GenericResponse::success(
        "Successfully deleted user.",
        (),
    )))
}

#[utoipa::path(
    patch,
    path = "/user/reactivate",
    tag = "User Account",
    description = "API for reactivation deleted User Account",
    summary = "User Account Reactivating API",
    request_body(content = SendOTPRequest, description = "Request Body"),
    responses(
        (status=200, description= "Successfully reactivated user.", body= GenericResponse<TupleUnit>),
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
    )
)]
#[tracing::instrument(err, name = "Delete User Account", skip(pool), fields())]
pub async fn reactivate_user_req(
    pool: web::Data<PgPool>,
    user_account: UserAccount,
) -> Result<web::Json<GenericResponse<()>>, GenericError> {
    if !user_account.is_deleted {
        return Err(GenericError::ValidationError(
            "User is not deleted".to_string(),
        ));
    }
    reactivate_user_account(&pool, user_account.id, user_account.id)
        .await
        .map_err(|e| {
            tracing::error!("Soft delete failed: {:?}", e);
            GenericError::DatabaseError("Failed to reactivate deleted user".to_string(), e)
        })?;
    Ok(web::Json(GenericResponse::success(
        "Successfully reactivated user.",
        (),
    )))
}

#[utoipa::path(
    patch,
    path = "/list",
    tag = "User Account",
    description = "API for listing users",
    summary = "List User Accounts API",
    request_body(content = ListUserAccountRequest, description = "Request Body"),
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
    )
)]
#[tracing::instrument(err, name = "List User Accounts", skip(pool), fields())]
pub async fn user_list_req(
    req: ListUserAccountRequest,
    pool: web::Data<PgPool>,
    user_account: UserAccount,
) -> Result<web::Json<GenericResponse<Vec<MinimalUserAccount>>>, GenericError> {
    let data = get_minimal_user_list(&pool, req.query.as_deref(), req.limit, req.offset, None)
        .await
        .map_err(|e| {
            tracing::error!("Soft delete failed: {:?}", e);
            GenericError::DatabaseError("Failed to fetch users".to_string(), e)
        })?;

    Ok(web::Json(GenericResponse::success(
        "Successfully fetched users.",
        data,
    )))
}

#[utoipa::path(
    patch,
    path = "/edit",
    tag = "User Account",
    description = "API for editing user account",
    summary = "List User Accounts API",
    request_body(content = EditUserAccount, description = "Request Body"),
    responses(
        (status=200, description= "Sucessfully fetched user data.", body= GenericResponse<TupleUnit>),
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
    )
)]
#[tracing::instrument(err, name = "User Account Edit", skip(pool), fields())]
pub async fn user_edit_req(
    data: EditUserAccount,
    pool: web::Data<PgPool>,
    user_account: UserAccount,
) -> Result<web::Json<GenericResponse<()>>, GenericError> {
    update_user(&pool, &data, &user_account)
        .await
        .map_err(|e| GenericError::UnexpectedCustomError(e.to_string()))?;
    Ok(web::Json(GenericResponse::success(
        "Successfully updated user.",
        (),
    )))
}

#[utoipa::path(
    post,
    path = "/user/password/reset",
    tag = "User Account",
    description = "API for resetting password",
    summary = "Password Reset API",
    request_body(content = PasswordResetReq, description = "Request Body"),
    responses(
        (status=200, description= "Sucessfully fetched user data.", body= GenericResponse<TupleUnit>),
        (status=400, description= "Invalid Request body", body= GenericResponse<TupleUnit>),
	    (status=403, description= "Insufficient Previlege", body= GenericResponse<TupleUnit>),
	    (status=410, description= "Data not found", body= GenericResponse<TupleUnit>),
        (status=500, description= "Internal Server Error", body= GenericResponse<TupleUnit>)
    ),
    params(
        ("x-request-id" = String, Header, description = "Request id"),
        ("x-device-id" = String, Header, description = "Device id"),
    )
)]
#[tracing::instrument(err, name = "User Password Reset", skip(pool), fields())]
pub async fn reset_password_req(
    data: PasswordResetReq,
    pool: web::Data<PgPool>,
    user_account: UserAccount,
) -> Result<web::Json<GenericResponse<()>>, GenericError> {
    reset_password(&pool, data.password, &user_account)
        .await
        .map_err(GenericError::UnexpectedError)?;
    Ok(web::Json(GenericResponse::success(
        "Successfully updated password.",
        (),
    )))
}
