use std::collections::HashSet;

use actix_web::web;
use sqlx::PgPool;
use tokio::join;
use utoipa::TupleUnit;
use uuid::Uuid;

use crate::{
    errors::GenericError,
    routes::{
        business::schemas::BusinessAccount, department::schemas::DepartmentAccount,
        role::utils::get_roles, user::schemas::UserAccount,
    },
    schemas::GenericResponse,
};

use super::{
    schemas::{Permission, PermissionLevel, PermissionRoleAssociationRequest},
    utils::{
        associate_permission_to_role, delete_role_permission_associations,
        fetch_permissions_by_scope,
    },
};

#[utoipa::path(
    get,
    description = "API for creating roles specific to business.",
    summary = "Business Permission List API",
    path = "/permission/business/list",
    tag = "Permission",

    responses(
        (status=200, description= "sucessfully listed business permissions", body= GenericResponse<Vec<Permission>>),
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
#[tracing::instrument(err, name = "Business Permission List API", skip(pool), fields())]
pub async fn list_business_permissions(
    pool: web::Data<PgPool>,
    user: UserAccount,
    business_account: BusinessAccount,
) -> Result<web::Json<GenericResponse<Vec<Permission>>>, GenericError> {
    let permissions = fetch_permissions_by_scope(&pool, vec![PermissionLevel::Business], None)
        .await
        .map_err(|e| {
            GenericError::DatabaseError(
                "Something went wrong while fetching business permissions".to_string(),
                e,
            )
        })?;
    Ok(web::Json(GenericResponse::success(
        "sucessfully listed business permissions",
        permissions,
    )))
}

#[utoipa::path(
    post,
    description = "API for associating permission to business roles.",
    summary = "Role Permission Association API",
    path = "/permission/business-role/associate",
    tag = "Permission",
    request_body(content = PermissionRoleAssociationRequest, description = "Request Body"),
    responses(
        (status=200, description= "sucessfully associated permissions to role", body= GenericResponse<TupleUnit>),
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
#[tracing::instrument(err, name = "Associate Permissions To Role API", skip(pool), fields())]
pub async fn associate_permissions_to_business_role(
    body: PermissionRoleAssociationRequest,
    pool: web::Data<PgPool>,
    user: UserAccount,
    business_account: BusinessAccount,
) -> Result<web::Json<GenericResponse<()>>, GenericError> {
    let permission_task = fetch_permissions_by_scope(
        &pool,
        vec![PermissionLevel::Business],
        Some(&body.permission_id_list),
    );
    let role_task = get_roles(
        &pool,
        Some(business_account.id),
        None,
        Some(vec![body.role_id]),
        None,
        true,
    );

    let (permission_res, role_res) = join!(permission_task, role_task);
    role_res
        .map_err(|e| GenericError::DatabaseError(e.to_string(), e))?
        .first()
        .ok_or_else(|| GenericError::DataNotFound("Role Not found.".to_string()))?;

    let permissions = permission_res.map_err(|e| GenericError::DatabaseError(e.to_string(), e))?;

    let found_permission_ids: HashSet<&Uuid> = permissions.iter().map(|p| &p.id).collect();

    // Find missing ones
    let requested_permission_ids: HashSet<&Uuid> = body.permission_id_list.iter().collect();
    let missing_permission_ids: Vec<&Uuid> = requested_permission_ids
        .difference(&found_permission_ids)
        .copied()
        .collect();

    if !missing_permission_ids.is_empty() {
        return Err(GenericError::ValidationError(format!(
            "Some permission IDs were not found: {:?}",
            missing_permission_ids
        )));
    }
    associate_permission_to_role(&pool, body.role_id, body.permission_id_list, user.id)
        .await
        .map_err(|e| {
            GenericError::DatabaseError(
                "Something went wrong while associating role to permission".to_string(),
                e,
            )
        })?;
    Ok(web::Json(GenericResponse::success(
        "sucessfully associated permissions to role",
        (),
    )))
}

#[utoipa::path(
    delete,
    description = "API for disassociating permission to business roles.",
    summary = "Business Role Permission Disassociation API",
    path = "/permission/business-role/disassociate",
    tag = "Permission",
    request_body(content = PermissionRoleAssociationRequest, description = "Request Body"),
    responses(
        (status=200, description= "sucessfully disassociated permissions to role", body= GenericResponse<TupleUnit>),
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
#[tracing::instrument(err, name = "Associate Permissions To Role API", skip(pool), fields())]
pub async fn disassociate_permissions_to_business_role(
    body: PermissionRoleAssociationRequest,
    pool: web::Data<PgPool>,
    user: UserAccount,
    business_account: BusinessAccount,
) -> Result<web::Json<GenericResponse<()>>, GenericError> {
    let permission_task = fetch_permissions_by_scope(
        &pool,
        vec![PermissionLevel::Business],
        Some(&body.permission_id_list),
    );
    let role_task = get_roles(
        &pool,
        Some(business_account.id),
        None,
        Some(vec![body.role_id]),
        None,
        true,
    );

    let (permission_res, role_res) = join!(permission_task, role_task);
    role_res
        .map_err(|e| GenericError::DatabaseError(e.to_string(), e))?
        .first()
        .ok_or_else(|| GenericError::DataNotFound("Role Not found.".to_string()))?;

    let permissions = permission_res.map_err(|e| GenericError::DatabaseError(e.to_string(), e))?;

    let found_permission_ids: HashSet<&Uuid> = permissions.iter().map(|p| &p.id).collect();

    // Find missing ones
    let requested_permission_ids: HashSet<&Uuid> = body.permission_id_list.iter().collect();

    let missing_permission_ids: Vec<&Uuid> = requested_permission_ids
        .difference(&found_permission_ids)
        .copied()
        .collect();

    if !missing_permission_ids.is_empty() {
        return Err(GenericError::ValidationError(format!(
            "Some permission IDs were not found: {:?}",
            missing_permission_ids
        )));
    }
    delete_role_permission_associations(&pool, body.permission_id_list, body.role_id)
        .await
        .map_err(|e| {
            GenericError::DatabaseError(
                "Something went wrong while disassociating role to permission".to_string(),
                e,
            )
        })?;
    Ok(web::Json(GenericResponse::success(
        "sucessfully disassociated permissions to role",
        (),
    )))
}

#[utoipa::path(
    get,
    description = "API for creating roles specific to department.",
    summary = "Department Permission List API",
    path = "/permission/department/list",
    tag = "Permission",

    responses(
        (status=200, description= "sucessfully listed business permissions", body= GenericResponse<Vec<Permission>>),
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
        ("x-department-id" = String, Header, description = "id of department_account"),
    )
)]
#[tracing::instrument(err, name = "Department Permission List API", skip(pool), fields())]
pub async fn list_department_permissions(
    pool: web::Data<PgPool>,
    user: UserAccount,
    business_account: BusinessAccount,
    department_account: DepartmentAccount,
) -> Result<web::Json<GenericResponse<Vec<Permission>>>, GenericError> {
    let permissions = fetch_permissions_by_scope(&pool, vec![PermissionLevel::Department], None)
        .await
        .map_err(|e| {
            GenericError::DatabaseError(
                "Something went wrong while fetching department permissions".to_string(),
                e,
            )
        })?;
    Ok(web::Json(GenericResponse::success(
        "sucessfully listed business permissions",
        permissions,
    )))
}

#[utoipa::path(
    post,
    description = "API for associating permission to department roles.",
    summary = "Deparment Role Permission Association API",
    path = "/permission/department-role/associate",
    tag = "Permission",
    request_body(content = PermissionRoleAssociationRequest, description = "Request Body"),
    responses(
        (status=200, description= "sucessfully associated permissions to role", body= GenericResponse<TupleUnit>),
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
        ("x-department-id" = String, Header, description = "id of department_account"),
      )
)]
#[tracing::instrument(
    err,
    name = "Associate Permissions To  Department Role API",
    skip(pool),
    fields()
)]
pub async fn associate_permissions_to_department_role(
    body: PermissionRoleAssociationRequest,
    pool: web::Data<PgPool>,
    user: UserAccount,
    business_account: BusinessAccount,
    department_account: DepartmentAccount,
) -> Result<web::Json<GenericResponse<()>>, GenericError> {
    let permission_task = fetch_permissions_by_scope(
        &pool,
        vec![PermissionLevel::Department],
        Some(&body.permission_id_list),
    );
    let role_task = get_roles(
        &pool,
        Some(business_account.id),
        Some(department_account.id),
        Some(vec![body.role_id]),
        None,
        true,
    );

    let (permission_res, role_res) = join!(permission_task, role_task);
    role_res
        .map_err(|e| GenericError::DatabaseError(e.to_string(), e))?
        .first()
        .ok_or_else(|| GenericError::DataNotFound("Role Not found.".to_string()))?;

    let permissions = permission_res.map_err(|e| GenericError::DatabaseError(e.to_string(), e))?;

    let found_permission_ids: HashSet<&Uuid> = permissions.iter().map(|p| &p.id).collect();

    // Find missing ones
    let requested_permission_ids: HashSet<&Uuid> = body.permission_id_list.iter().collect();
    let missing_permission_ids: Vec<&Uuid> = requested_permission_ids
        .difference(&found_permission_ids)
        .copied()
        .collect();

    if !missing_permission_ids.is_empty() {
        return Err(GenericError::ValidationError(format!(
            "Some permission IDs were not found: {:?}",
            missing_permission_ids
        )));
    }
    associate_permission_to_role(&pool, body.role_id, body.permission_id_list, user.id)
        .await
        .map_err(|e| {
            GenericError::DatabaseError(
                "Something went wrong while associating role to permission".to_string(),
                e,
            )
        })?;
    Ok(web::Json(GenericResponse::success(
        "sucessfully associated permissions to role",
        (),
    )))
}

#[utoipa::path(
    delete,
    description = "API for disassociating permission to department roles.",
    summary = "Department Role Permission Disassociation API",
    path = "/permission/department-role/disassociate",
    tag = "Permission",
    request_body(content = PermissionRoleAssociationRequest, description = "Request Body"),
    responses(
        (status=200, description= "sucessfully disassociated permissions to role", body= GenericResponse<TupleUnit>),
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
        ("x-department-id" = String, Header, description = "id of department_account"),
      )
)]
#[tracing::instrument(err, name = "Associate Permissions To Role API", skip(pool), fields())]
pub async fn disassociate_permissions_to_department_role(
    body: PermissionRoleAssociationRequest,
    pool: web::Data<PgPool>,
    user: UserAccount,
    business_account: BusinessAccount,
    department_account: DepartmentAccount,
) -> Result<web::Json<GenericResponse<()>>, GenericError> {
    let permission_task = fetch_permissions_by_scope(
        &pool,
        vec![PermissionLevel::Business],
        Some(&body.permission_id_list),
    );
    let role_task = get_roles(
        &pool,
        Some(business_account.id),
        Some(department_account.id),
        Some(vec![body.role_id]),
        None,
        true,
    );

    let (permission_res, role_res) = join!(permission_task, role_task);
    role_res
        .map_err(|e| GenericError::DatabaseError(e.to_string(), e))?
        .first()
        .ok_or_else(|| GenericError::DataNotFound("Role Not found.".to_string()))?;

    let permissions = permission_res.map_err(|e| GenericError::DatabaseError(e.to_string(), e))?;

    let found_permission_ids: HashSet<&Uuid> = permissions.iter().map(|p| &p.id).collect();

    // Find missing ones
    let requested_permission_ids: HashSet<&Uuid> = body.permission_id_list.iter().collect();

    let missing_permission_ids: Vec<&Uuid> = requested_permission_ids
        .difference(&found_permission_ids)
        .copied()
        .collect();

    if !missing_permission_ids.is_empty() {
        return Err(GenericError::ValidationError(format!(
            "Some permission IDs were not found: {:?}",
            missing_permission_ids
        )));
    }
    delete_role_permission_associations(&pool, body.permission_id_list, body.role_id)
        .await
        .map_err(|e| {
            GenericError::DatabaseError(
                "Something went wrong while disassociating role to permission".to_string(),
                e,
            )
        })?;
    Ok(web::Json(GenericResponse::success(
        "sucessfully disassociated permissions to role",
        (),
    )))
}
