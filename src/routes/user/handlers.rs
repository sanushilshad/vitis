use actix_web::web;
use sqlx::PgPool;
use utoipa::TupleUnit;

use crate::{
    configuration::{SecretConfig, UserConfig},
    errors::GenericError,
    schemas::{DeleteType, GenericResponse, RequestMetaData, Status},
};

use super::{
    errors::{AuthError, UserRegistrationError},
    schemas::{
        AuthData, AuthenticateRequest, AuthenticationScope, CreateUserAccount, SendOTPRequest,
        UserAccount, UserType, VectorType,
    },
    utils::{
        fetch_user, get_auth_data, get_stored_credentials, hard_delete_user_account,
        reactivate_user_account, register_user, send_otp, soft_delete_user_account,
        update_user_verification_status, validate_user_credentials,
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
    let admin_role = [UserType::Admin, UserType::Superadmin];
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
        .await?
        .ok_or_else(|| {
            GenericError::UnexpectedCustomError(
                "Something went wrong while fetching user data".to_string(),
            )
        })?;

    let user_account = user_model.into_schema();

    if !user_account.is_vector_verified(VectorType::MobileNo) {
        if body.scope != AuthenticationScope::Otp {
            return Err(GenericError::UnAuthorized(
                "Please Verify your mobile no".to_string(),
            ));
        } else {
            update_user_verification_status(&pool, VectorType::MobileNo, user_id, true)
                .await
                .map_err(|_| {
                    AuthError::UnexpectedCustomError(
                        "Something went wrong while updating user data".to_string(),
                    )
                })?;
        }
    }

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
    )
)]
#[tracing::instrument(err, name = "send OTP", skip(pool), fields())]
pub async fn send_otp_req(
    req: SendOTPRequest,
    pool: web::Data<PgPool>,
    secret_obj: web::Data<SecretConfig>,
) -> Result<web::Json<GenericResponse<()>>, GenericError> {
    let credential = get_stored_credentials(&req.mobile_no, &AuthenticationScope::Otp, &pool)
        .await
        .map_err(|_| {
            GenericError::UnexpectedCustomError(
                "Something went wrong while fetching auth details".to_string(),
            )
        })?
        .ok_or_else(|| GenericError::UnexpectedCustomError("User not found".to_string()))?;
    if credential.is_active == Status::Inactive {
        return Err(GenericError::UnexpectedCustomError(
            "Auth Mechanism is disabled".to_string(),
        ));
    }
    send_otp(&pool, "000", secret_obj.otp.expiry, credential)
        .await
        .map_err(|_| {
            GenericError::UnexpectedCustomError(
                "Something went wrong while sending OTP".to_string(),
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
        (status=200, description= "Sucessfully fetched user data.", body= GenericResponse<TupleUnit>),
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
                GenericError::UnexpectedCustomError("Failed to soft delete user".to_string())
            })?;
        }
        DeleteType::Hard => {
            hard_delete_user_account(pool.get_ref(), identifier)
                .await
                .map_err(|e| {
                    tracing::error!("Hard delete failed: {:?}", e);
                    GenericError::UnexpectedCustomError("Failed to hard delete user".to_string())
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
    description = "API for reactivating deleted User Account",
    summary = "User Account Reactivating API",
    request_body(content = SendOTPRequest, description = "Request Body"),
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
    )
)]
#[tracing::instrument(err, name = "Delete User Account", skip(pool), fields())]
pub async fn reactivate_user_req(
    pool: web::Data<PgPool>,
    user_account: UserAccount,
) -> Result<web::Json<GenericResponse<()>>, GenericError> {
    if user_account.is_deleted == false {
        return Err(GenericError::ValidationError(
            "User is not deleted".to_string(),
        ));
    }
    let _ = reactivate_user_account(&pool, user_account.id, user_account.id)
        .await
        .map_err(|e| {
            tracing::error!("Soft delete failed: {:?}", e);
            GenericError::UnexpectedCustomError("Failed to reactivate deleted user".to_string())
        })?;
    Ok(web::Json(GenericResponse::success(
        "Successfully reactivated user.",
        (),
    )))
}
