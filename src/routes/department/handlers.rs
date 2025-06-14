use actix::Addr;
use actix_web::web;
use sqlx::PgPool;
use utoipa::TupleUnit;

use crate::{
    errors::GenericError,
    routes::user::{
        schemas::{RoleType, UserAccount},
        utils::get_role,
    },
    schemas::{GenericResponse, RequestMetaData},
    websocket::{MessageToClient, Server, WebSocketActionType, WebSocketData},
};

use super::{
    schemas::{
        BasicDepartmentAccount, CreateDepartmentAccount, DepartmentAccount, DepartmentFetchRequest,
        DepartmentUserAssociationRequest, departmentPermissionRequest,
    },
    utils::{
        associate_user_to_department, create_department_account, get_basic_department_accounts,
        get_basic_department_accounts_by_user_id, get_department_account,
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
    description = "API for creating department accounts. A single user can have multiple department accounts",
    summary = "Department Account Registration API",
    path = "/department/register",
    tag = "department Account",
    request_body(content = CreateDepartmentAccount, description = "Request Body"),
    responses(
        (status=200, description= "department Account created successfully", body= GenericResponse<TupleUnit>),
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
    name = "department Account Registration API",
    skip(pool, body),
    fields()
)]
pub async fn register_department_account_req(
    body: CreateDepartmentAccount,
    pool: web::Data<PgPool>,
    meta_data: RequestMetaData,
    user: UserAccount,
) -> Result<web::Json<GenericResponse<()>>, GenericError> {
    create_department_account(&pool, &user, &body).await?;
    Ok(web::Json(GenericResponse::success(
        "Sucessfully Registered department Account.",
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
      )
)]
#[tracing::instrument(err, name = "fetch department detail", skip(db_pool), fields())]
pub async fn fetch_department_req(
    db_pool: web::Data<PgPool>,
    user_account: UserAccount,
    body: DepartmentFetchRequest,
) -> Result<web::Json<GenericResponse<DepartmentAccount>>, GenericError> {
    let department_account = get_department_account(&db_pool, user_account.id, body.id)
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
    request_body(content = departmentPermissionRequest, description = "Request Body"),
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
#[tracing::instrument(err, name = "department Permission", skip(db_pool), fields())]
pub async fn department_permission_validation(
    db_pool: web::Data<PgPool>,
    user_account: UserAccount,
    body: departmentPermissionRequest,
) -> Result<web::Json<GenericResponse<Vec<String>>>, GenericError> {
    let permission_list = validate_user_department_permission(
        &db_pool,
        user_account.id,
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
        (status=200, description= "Sucessfully fetched department data.", body= GenericResponse<Vec<BasicDepartmentAccount>>),
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
#[tracing::instrument(err, name = "fetch department detail", skip(db_pool), fields())]
pub async fn list_department_req(
    db_pool: web::Data<PgPool>,
    user_account: UserAccount,
) -> Result<web::Json<GenericResponse<Vec<BasicDepartmentAccount>>>, GenericError> {
    let department_obj = if user_account.user_role != RoleType::Superadmin.to_string() {
        get_basic_department_accounts_by_user_id(user_account.id, &db_pool).await
    } else {
        get_basic_department_accounts(&db_pool).await
    }
    .map_err(|e| GenericError::DatabaseError(e.to_string(), e))?;
    Ok(web::Json(GenericResponse::success(
        "Sucessfully fetched all associated department accounts.",
        department_obj,
    )))
}

#[utoipa::path(
    post,
    path = "/user/assocation",
    tag = "department Account",
    description = "API for association of user with department account",
    summary = "Use department Account Association API",
    // request_body(content = departmentAccountListReq, description = "Request Body"),
    responses(
        (status=200, description= "Sucessfully fetched department data.", body= GenericResponse<TupleUnit>),
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
#[tracing::instrument(err, name = "user department association", skip(db_pool), fields())]
pub async fn user_department_association_req(
    req: DepartmentUserAssociationRequest,
    db_pool: web::Data<PgPool>,
    user_account: UserAccount,
    // department_account: DepartmentAccount,
    websocket_srv: web::Data<Addr<Server>>,
) -> Result<web::Json<GenericResponse<()>>, GenericError> {
    if req.role == RoleType::Superadmin {
        return Err(GenericError::InsufficientPrevilegeError(
            "Insufficient previlege to assign Superadmin".to_string(),
        ));
    }
    let role_obj_task = get_role(&db_pool, &req.role);

    let assocated_user_task = get_department_account(&db_pool, user_account.id, req.department_id);

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
        req.department_id,
        role_obj.id,
        user_account.id,
    )
    .await
    .map_err(|_| {
        GenericError::UnexpectedCustomError(
            "Something went wrong while associating user to department".to_owned(),
        )
    })?;
    let msg: MessageToClient = MessageToClient::new(
        WebSocketActionType::UserDepartmentAssociation,
        serde_json::to_value(WebSocketData {
            message: "Successfully associated user".to_string(),
        })
        .unwrap(),
        Some(user_account.id),
        None,
        None,
    );
    websocket_srv.do_send(msg);
    Ok(web::Json(GenericResponse::success(
        "Sucessfully associated user with department account.",
        (),
    )))
}
