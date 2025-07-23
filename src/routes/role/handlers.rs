use actix_web::web;
use chrono::Utc;
use sqlx::PgPool;
use utoipa::TupleUnit;
use uuid::Uuid;

use crate::{
    errors::GenericError,
    routes::{
        business::schemas::BusinessAccount,
        permission::{schemas::Permission, utils::fetch_permissions_for_role},
        user::schemas::UserAccount,
    },
    schemas::GenericResponse,
};

use super::{
    schemas::{AccountRole, CreateBusinessRoleRequest},
    utils::{get_roles, save_role, soft_delete_role},
};

#[utoipa::path(
    post,
    description = "API for creating roles specific to business.",
    summary = "Business Roles Create API",
    path = "/role/business/save",
    tag = "Role",
    request_body(content = CreateBusinessRoleRequest, description = "Request Body"),
    responses(
        (status=200, description= "sucessfully created business roles", body= GenericResponse<TupleUnit>),
        (status=400, description= "Invalid Request body", body= GenericResponse<TupleUnit>),
        (status=401, description= "Invalid Token", body= GenericResponse<TupleUnit>),
	    (status=403, description= "Insufficient Previlege", body= GenericResponse<TupleUnit>),
	    (status=410, description= "Data not found", body= GenericResponse<TupleUnit>),
        (status=500, description= "Internal Server Error", body= GenericResponse<TupleUnit>)
    ),
    params(
        ("Authorization" = String, Header, description = "JWT token"),
        ("x-business-id" = String, Header, description = "id of business_account"),
        ("x-request-id" = String, Header, description = "Request id"),
        ("x-device-id" = String, Header, description = "Device id"),
      )
)]
#[tracing::instrument(err, name = "Business role Creation API", skip(pool), fields())]
pub async fn save_business_role_req(
    body: CreateBusinessRoleRequest,
    pool: web::Data<PgPool>,
    user: UserAccount,
    business_account: BusinessAccount,
) -> Result<web::Json<GenericResponse<()>>, GenericError> {
    let new_role_names: Vec<&str> = body
        .data
        .iter()
        .filter(|a| a.id.is_none())
        .map(|f| f.name.as_ref())
        .collect();
    if !new_role_names.is_empty() {
        let roles = get_roles(
            &pool,
            Some(business_account.id),
            None,
            None,
            Some(new_role_names),
            true,
        )
        .await
        .map_err(|e| GenericError::DatabaseError(e.to_string(), e))?;
        if !roles.is_empty() {
            let existing_labels = roles
                .iter()
                .map(|p| p.name.as_str())
                .collect::<Vec<&str>>()
                .join(", ");

            return Err(GenericError::ValidationError(format!(
                "Roles already exist: {}",
                existing_labels
            )));
        }
    }

    save_role(
        &pool,
        &body.data,
        Some(business_account.id),
        None,
        user.id,
        Utc::now(),
    )
    .await
    .map_err(|e| GenericError::UnexpectedCustomError(e.to_string()))?;
    Ok(web::Json(GenericResponse::success(
        "sucessfully created business roles",
        (),
    )))
}

#[utoipa::path(
    get,
    description = "API for listing roles specific to business.",
    summary = "Business Roles Listing API",
    path = "/role/business/list",
    tag = "Role",
    // request_body(content = CreateBusinessSettingRequest, description = "Request Body"),
    responses(
        (status=200, description= "sucessfully listed business roles", body= GenericResponse<Vec<AccountRole>>),
        (status=400, description= "Invalid Request body", body= GenericResponse<TupleUnit>),
        (status=401, description= "Invalid Token", body= GenericResponse<TupleUnit>),
	    (status=403, description= "Insufficient Previlege", body= GenericResponse<TupleUnit>),
	    (status=410, description= "Data not found", body= GenericResponse<TupleUnit>),
        (status=500, description= "Internal Server Error", body= GenericResponse<TupleUnit>)
    ),
    params(
        ("Authorization" = String, Header, description = "JWT token"),
        ("x-business-id" = String, Header, description = "id of business_account"),
        ("x-request-id" = String, Header, description = "Request id"),
        ("x-device-id" = String, Header, description = "Device id"),
      )
)]
#[tracing::instrument(err, name = "Business Role List API", skip(pool), fields())]
pub async fn list_business_role_req(
    // body: CreateBusinessSettingRequest,
    pool: web::Data<PgPool>,
    user: UserAccount,
    business_account: BusinessAccount,
) -> Result<web::Json<GenericResponse<Vec<AccountRole>>>, GenericError> {
    let roles = get_roles(&pool, Some(business_account.id), None, None, None, true)
        .await
        .map_err(|e| GenericError::DatabaseError(e.to_string(), e))?;
    Ok(web::Json(GenericResponse::success(
        "sucessfully listed business roles",
        roles,
    )))
}

#[utoipa::path(
    delete,
    description = "API for deleting roles specific to business.",
    summary = "Business Roles Deletion API",
    path = "/role/business/delete/{id}",
    tag = "Role",
    // request_body(content = CreateBusinessSettingRequest, description = "Request Body"),
    responses(
        (status=200, description= "sucessfully delete business role", body= GenericResponse<TupleUnit>),
        (status=400, description= "Invalid Request body", body= GenericResponse<TupleUnit>),
        (status=401, description= "Invalid Token", body= GenericResponse<TupleUnit>),
	    (status=403, description= "Insufficient Previlege", body= GenericResponse<TupleUnit>),
	    (status=410, description= "Data not found", body= GenericResponse<TupleUnit>),
        (status=500, description= "Internal Server Error", body= GenericResponse<TupleUnit>)
    ),
    params(
        ("Authorization" = String, Header, description = "JWT token"),
        ("x-business-id" = String, Header, description = "id of business_account"),
        ("x-request-id" = String, Header, description = "Request id"),
        ("x-device-id" = String, Header, description = "Device id"),
        ("id" = String, Path, description = "Role ID"),
      )
)]
#[tracing::instrument(err, name = "Business Role List API", skip(pool), fields())]
pub async fn delete_business_role_req(
    path: web::Path<Uuid>,
    pool: web::Data<PgPool>,
    user: UserAccount,
    business_account: BusinessAccount,
) -> Result<web::Json<GenericResponse<()>>, GenericError> {
    let role_id = path.into_inner();
    let roles = get_roles(
        &pool,
        Some(business_account.id),
        None,
        Some(vec![role_id]),
        None,
        false,
    )
    .await
    .map_err(|e| GenericError::DatabaseError(e.to_string(), e))?;
    if roles.is_empty() {
        return Err(GenericError::ValidationError("Invalid Role ID".to_string()));
    }
    soft_delete_role(&pool, business_account.id, role_id, user.id, Utc::now())
        .await
        .map_err(|e| GenericError::DatabaseError(e.to_string(), e))?;
    Ok(web::Json(GenericResponse::success(
        "sucessfully delete business role",
        (),
    )))
}

#[utoipa::path(
    get,
    description = "API for listing permissions associated to roles",
    summary = "Business Roles Permission List API",
    path = "/role/business-permission/list/{id}",
    tag = "Role",
    // request_body(content = CreateBusinessSettingRequest, description = "Request Body"),
    responses(
        (status=200, description= "sucessfully listed role permissions", body= GenericResponse<Vec<Permission>>),
        (status=400, description= "Invalid Request body", body= GenericResponse<TupleUnit>),
        (status=401, description= "Invalid Token", body= GenericResponse<TupleUnit>),
	    (status=403, description= "Insufficient Previlege", body= GenericResponse<TupleUnit>),
	    (status=410, description= "Data not found", body= GenericResponse<TupleUnit>),
        (status=500, description= "Internal Server Error", body= GenericResponse<TupleUnit>)
    ),
    params(
        ("Authorization" = String, Header, description = "JWT token"),
        ("x-business-id" = String, Header, description = "id of business_account"),
        ("x-request-id" = String, Header, description = "Request id"),
        ("x-device-id" = String, Header, description = "Device id"),
        ("id" = String, Path, description = "Role ID"),
      )
)]
#[tracing::instrument(err, name = "Business Role List API", skip(pool), fields())]
pub async fn list_role_permission_list_req(
    path: web::Path<Uuid>,
    pool: web::Data<PgPool>,
    user: UserAccount,
    business_account: BusinessAccount,
) -> Result<web::Json<GenericResponse<Vec<Permission>>>, GenericError> {
    let role_id = path.into_inner();
    let roles = get_roles(&pool, None, None, Some(vec![role_id]), None, false)
        .await
        .map_err(|e| GenericError::DatabaseError(e.to_string(), e))?;
    if roles.is_empty() {
        return Err(GenericError::ValidationError("Invalid Role ID".to_string()));
    }
    let permissions = fetch_permissions_for_role(&pool, role_id, business_account.id)
        .await
        .map_err(|e| GenericError::DatabaseError(e.to_string(), e))?;
    Ok(web::Json(GenericResponse::success(
        "sucessfully listed role permissions",
        permissions,
    )))
}

#[utoipa::path(
    post,
    description = "API for creating roles specific to department.",
    summary = "Bepartment Roles Create API",
    path = "/role/department/save",
    tag = "Role",
    request_body(content = CreateBusinessRoleRequest, description = "Request Body"),
    responses(
        (status=200, description= "sucessfully created business roles", body= GenericResponse<TupleUnit>),
        (status=400, description= "Invalid Request body", body= GenericResponse<TupleUnit>),
        (status=401, description= "Invalid Token", body= GenericResponse<TupleUnit>),
	    (status=403, description= "Insufficient Previlege", body= GenericResponse<TupleUnit>),
	    (status=410, description= "Data not found", body= GenericResponse<TupleUnit>),
        (status=500, description= "Internal Server Error", body= GenericResponse<TupleUnit>)
    ),
    params(
        ("Authorization" = String, Header, description = "JWT token"),
        ("x-business-id" = String, Header, description = "id of business_account"),
        ("x-request-id" = String, Header, description = "Request id"),
        ("x-device-id" = String, Header, description = "Device id"),
        ("x-department-id" = String, Header, description = "id of department account"), 
      )
)]
#[tracing::instrument(err, name = "Business role Creation API", skip(pool), fields())]
pub async fn save_department_role_req(
    body: CreateBusinessRoleRequest,
    pool: web::Data<PgPool>,
    user: UserAccount,
    business_account: BusinessAccount,
    department_account: BusinessAccount,
) -> Result<web::Json<GenericResponse<()>>, GenericError> {
    let new_role_names: Vec<&str> = body
        .data
        .iter()
        .filter(|a| a.id.is_none())
        .map(|f| f.name.as_ref())
        .collect();
    if !new_role_names.is_empty() {
        let roles = get_roles(
            &pool,
            Some(business_account.id),
            Some(department_account.id),
            None,
            Some(new_role_names),
            true,
        )
        .await
        .map_err(|e| GenericError::DatabaseError(e.to_string(), e))?;
        if !roles.is_empty() {
            let existing_labels = roles
                .iter()
                .map(|p| p.name.as_str())
                .collect::<Vec<&str>>()
                .join(", ");

            return Err(GenericError::ValidationError(format!(
                "Roles already exist: {}",
                existing_labels
            )));
        }
    }

    save_role(
        &pool,
        &body.data,
        Some(business_account.id),
        Some(department_account.id),
        user.id,
        Utc::now(),
    )
    .await
    .map_err(|e| GenericError::UnexpectedCustomError(e.to_string()))?;
    Ok(web::Json(GenericResponse::success(
        "sucessfully created business roles",
        (),
    )))
}
