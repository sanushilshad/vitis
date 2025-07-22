use actix::Addr;
use actix_web::web;
use chrono::Utc;
use sqlx::PgPool;
use utoipa::TupleUnit;

use crate::{
    errors::GenericError,
    pulsar_client::PulsarClient,
    routes::{
        business::schemas::BusinessAccount,
        role::utils::get_role,
        user::schemas::{UserAccount, UserRoleType},
        web_socket::{schemas::ProcessType, utils::send_notification},
    },
    schemas::{AllowedPermission, GenericResponse, PermissionType, RequestMetaData},
    websocket_client::{Server, WebSocketActionType},
};

use super::{
    schemas::{
        BasicDepartmentAccount, CreateDepartmentAccount, DepartmentAccount, DepartmentFetchRequest,
        DepartmentPermissionRequest, DepartmentUserAssociationRequest, UpdateDepartmentAccount,
        UserDepartmentDeassociationRequest,
    },
    utils::{
        associate_user_to_department, create_department_account,
        delete_user_department_relationship, get_basic_department_accounts,
        get_basic_department_accounts_by_user_id, get_department_account,
        soft_delete_department_account, update_department_account,
        validate_user_department_permission,
    },
    // utils::{
    //     associate_user_to_department, create_department_account, get_basic_department_accounts,
    //     get_basic_department_accounts_by_user_id, get_department_account,
    //     validate_user_department_permission,
    // },
};

#[utoipa::path(
    post,
    description = "API for creating department accounts. A business account can have multiple department accounts",
    summary = "Department Account Registration API",
    path = "/department/register",
    tag = "department Account",
    request_body(content = CreateDepartmentAccount, description = "Request Body"),
    responses(
        (status=200, description= "sucessfully registered department Account.", body= GenericResponse<TupleUnit>),
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
    name = "department Account Registration API",
    skip(pool, body),
    fields()
)]
pub async fn register_department_account_req(
    body: CreateDepartmentAccount,
    pool: web::Data<PgPool>,
    meta_data: RequestMetaData,
    user: UserAccount,
    business_account: BusinessAccount,
) -> Result<web::Json<GenericResponse<()>>, GenericError> {
    create_department_account(&pool, &user, &business_account, &body).await?;
    Ok(web::Json(GenericResponse::success(
        "sucessfully registered department Account.",
        (),
    )))
}

#[utoipa::path(
    post,
    path = "/department/fetch",
    tag = "department Account",
    description = "API for fetching department account detail.",
    summary = "department Account Fetch API",
    request_body(content =DepartmentFetchRequest, description = "Request Body"),
    responses(
        (status=200, description= "Sucessfully fetched department data.", body= GenericResponse<DepartmentAccount>),
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
#[tracing::instrument(err, name = "fetch department detail", skip(db_pool), fields())]
pub async fn fetch_department_req(
    db_pool: web::Data<PgPool>,
    user_account: UserAccount,
    business_account: BusinessAccount,
    body: DepartmentFetchRequest,
) -> Result<web::Json<GenericResponse<DepartmentAccount>>, GenericError> {
    let department_account =
        get_department_account(&db_pool, user_account.id, business_account.id, body.id)
            .await
            .map_err(|e| GenericError::DatabaseError(e.to_string(), e))?
            .ok_or_else(|| {
                GenericError::ValidationError("department account does not exist.".to_string())
            })?;
    Ok(web::Json(GenericResponse::success(
        "Sucessfully fetched department data.",
        department_account,
    )))
}

#[utoipa::path(
    post,
    path = "/department/permission",
    tag = "department Account",
    description = "API for checking the permission of a department.",
    summary = "department Account Permission API",
    request_body(content = DepartmentPermissionRequest, description = "Request Body"),
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
        ("x-business-id" = String, Header, description = "id of business_account"),
      )
)]
#[tracing::instrument(err, name = "department Permission", skip(db_pool), fields())]
pub async fn department_permission_validation(
    db_pool: web::Data<PgPool>,
    user_account: UserAccount,
    business_account: BusinessAccount,
    body: DepartmentPermissionRequest,
) -> Result<web::Json<GenericResponse<Vec<String>>>, GenericError> {
    let permission_list = validate_user_department_permission(
        &db_pool,
        user_account.id,
        business_account.id,
        body.department_id,
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
    path = "/department/list",
    tag = "department Account",
    description = "API for listing all department account associated to a user",
    summary = "department Account List API",
    // request_body(content = departmentAccountListReq, description = "Request Body"),
    responses(
        (status=200, description= "sucessfully fetched all associated department accounts.", body= GenericResponse<Vec<BasicDepartmentAccount>>),
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
#[tracing::instrument(err, name = "fetch department detail", skip(db_pool), fields())]
pub async fn list_department_req(
    db_pool: web::Data<PgPool>,
    user_account: UserAccount,
    business_account: BusinessAccount,
) -> Result<web::Json<GenericResponse<Vec<BasicDepartmentAccount>>>, GenericError> {
    let department_obj = if user_account.user_role != UserRoleType::Superadmin.to_string() {
        get_basic_department_accounts_by_user_id(user_account.id, business_account.id, &db_pool)
            .await
    } else {
        get_basic_department_accounts(&db_pool).await
    }
    .map_err(|e| GenericError::DatabaseError(e.to_string(), e))?;
    Ok(web::Json(GenericResponse::success(
        "sucessfully fetched all associated department accounts.",
        department_obj,
    )))
}

#[utoipa::path(
    post,
    path = "/user/assocation",
    tag = "department Account",
    description = "API for association of user with a department account",
    summary = "Use department Account Association API",
    // request_body(content = departmentAccountListReq, description = "Request Body"),
    responses(
        (status=200, description= "sucessfully associated user with department account.", body= GenericResponse<TupleUnit>),
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
        ("x-department-id" = String, Header, description = "id of department account"),
      )
)]
#[tracing::instrument(
    err,
    name = "user department association",
    skip(db_pool, producer_client),
    fields()
)]
pub async fn user_department_association_req(
    req: DepartmentUserAssociationRequest,
    db_pool: web::Data<PgPool>,
    user_account: UserAccount,
    business_account: BusinessAccount,
    department_account: DepartmentAccount,
    websocket_srv: web::Data<Addr<Server>>,
    producer_client: web::Data<PulsarClient>,
) -> Result<web::Json<GenericResponse<()>>, GenericError> {
    let role = req.role.to_lowercase_string();
    let role_obj_task = get_role(&db_pool, &role);

    let assocated_user_task = get_department_account(
        &db_pool,
        user_account.id,
        business_account.id,
        department_account.id,
    );

    let (role_obj_res, assocated_user_res) = tokio::join!(role_obj_task, assocated_user_task);

    let assocated_user = assocated_user_res.map_err(|e| {
        GenericError::DatabaseError(
            "Something went wrong while fetching existing user-department association".to_owned(),
            e,
        )
    })?;

    let role_obj = role_obj_res
        .map_err(|e| {
            GenericError::DatabaseError("Something went wrong while fetching role".to_string(), e)
        })?
        .ok_or_else(|| GenericError::ValidationError("role does not exist.".to_string()))?;

    if assocated_user.is_some() {
        return Err(GenericError::ValidationError(
            "User already associated with department".to_owned(),
        ));
    }
    associate_user_to_department(
        &db_pool,
        req.user_id,
        business_account.id,
        department_account.id,
        role_obj.id,
        user_account.id,
    )
    .await
    .map_err(|_| {
        GenericError::UnexpectedCustomError(
            "Something went wrong while associating user to department".to_owned(),
        )
    })?;
    send_notification(
        &db_pool,
        &websocket_srv,
        WebSocketActionType::UserDepartmentAssociation,
        ProcessType::Deferred,
        vec![req.user_id],
        format!(
            "{} Department is associated to you account by {}",
            department_account.display_name, user_account.display_name
        ),
        Some(business_account.id),
        &producer_client,
    )
    .await
    .map_err(|e| GenericError::UnexpectedCustomError(e.to_string()))?;
    Ok(web::Json(GenericResponse::success(
        "sucessfully associated user with department account.",
        (),
    )))
}

#[utoipa::path(
    post,
    path = "/department/user/disassociate",
    tag = "Department Account",
    description = "API for disassociating user froms business account",
    summary = "Use business Account Disassociation API",
    request_body(content = UserDepartmentDeassociationRequest, description = "Request Body"),
    responses(
        (status=200, description= "sucessfully disassociated user from department account.", body= GenericResponse<TupleUnit>),
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
        ("x-department-id" = String, Header, description = "id of department account"),    
      )
)]
#[tracing::instrument(err, name = "user business disassociation", skip(pool), fields())]
pub async fn user_department_deassociation_req(
    req: UserDepartmentDeassociationRequest,
    pool: web::Data<PgPool>,
    user_account: UserAccount,
    business_account: BusinessAccount,
    department_account: DepartmentAccount,
    permissions: AllowedPermission,
) -> Result<web::Json<GenericResponse<()>>, GenericError> {
    let user_id = req
        .id
        .filter(|_| {
            !permissions
                .permission_list
                .contains(&PermissionType::DisassociateBusiness.to_string())
        })
        .unwrap_or(user_account.id);
    delete_user_department_relationship(&pool, user_id, business_account.id, department_account.id)
        .await
        .map_err(|e| {
            GenericError::DatabaseError(
                "Something went wrong while disassociating user from department".to_owned(),
                e,
            )
        })?;
    Ok(web::Json(GenericResponse::success(
        "sucessfully disassociated user from department account.",
        (),
    )))
}

#[utoipa::path(
    delete,
    path = "/department/delete",
    tag = "Department Account",
    description = "API for deleting  department account",
    summary = "Department Account Deletion API",
    responses(
        (status=200, description= "sucessfully deleted department account.", body= GenericResponse<TupleUnit>),
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
        ("x-department-id" = String, Header, description = "id of department account"), 
      )
)]
#[tracing::instrument(err, name = "user department disassociation", skip(pool), fields())]
pub async fn department_account_deletion_req(
    pool: web::Data<PgPool>,
    user_account: UserAccount,
    business_account: BusinessAccount,
    department_account: DepartmentAccount,
) -> Result<web::Json<GenericResponse<()>>, GenericError> {
    soft_delete_department_account(&pool, department_account.id, user_account.id, Utc::now())
        .await
        .map_err(|e| {
            GenericError::DatabaseError(
                "Something went wrong while deleting department account".to_owned(),
                e,
            )
        })?;
    Ok(web::Json(GenericResponse::success(
        "sucessfully deleted  department account.",
        (),
    )))
}

#[utoipa::path(
    patch,
    path = "/department/update",
    tag = "Department Account",
    description = "API for updating  department account",
    summary = "Department Account Updation API",
    responses(
        (status=200, description= "sucessfully updated department account.", body= GenericResponse<TupleUnit>),
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
        ("x-department-id" = String, Header, description = "id of department account"), 
      )
)]
#[tracing::instrument(err, name = "user department updation", skip(pool), fields())]
pub async fn department_account_updation_req(
    pool: web::Data<PgPool>,
    user_account: UserAccount,
    business_account: BusinessAccount,
    department_account: DepartmentAccount,
    req: UpdateDepartmentAccount,
) -> Result<web::Json<GenericResponse<()>>, GenericError> {
    update_department_account(&pool, &req, &department_account, user_account.id)
        .await
        .map_err(|e| {
            GenericError::DatabaseError(
                "Something went wrong while updating department".to_owned(),
                e,
            )
        })?;

    Ok(web::Json(GenericResponse::success(
        "sucessfully updated department account.",
        (),
    )))
}
