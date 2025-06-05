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
        BasicprojectAccount, CreateprojectAccount, ProjectAccount, ProjectFetchRequest,
        ProjectPermissionRequest, ProjectUserAssociationRequest,
    },
    utils::{
        associate_user_to_project, create_project_account, get_basic_project_accounts,
        get_basic_project_accounts_by_user_id, get_project_account,
        validate_user_project_permission,
    },
};

#[utoipa::path(
    post,
    description = "API for creating project accounts for a user. A single user can have multiple project accounts",
    summary = "project Account Registration API",
    path = "/project/register",
    tag = "project Account",
    request_body(content = CreateprojectAccount, description = "Request Body"),
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
    name = "project Account Registration API",
    skip(pool, body),
    fields()
)]
pub async fn register_project_account_req(
    body: CreateprojectAccount,
    pool: web::Data<PgPool>,
    meta_data: RequestMetaData,
    user: UserAccount,
) -> Result<web::Json<GenericResponse<()>>, GenericError> {
    create_project_account(&pool, &user, &body).await?;
    Ok(web::Json(GenericResponse::success(
        "Sucessfully Registered project Account.",
        (),
    )))
}

#[utoipa::path(
    post,
    path = "/project/fetch",
    tag = "project Account",
    description = "API for fetching project account detail.",
    summary = "project Account Fetch API",
    request_body(content = ProjectFetchRequest, description = "Request Body"),
    responses(
        (status=200, description= "Sucessfully fetched project data.", body= GenericResponse<ProjectAccount>),
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
#[tracing::instrument(err, name = "fetch project detail", skip(db_pool), fields())]
pub async fn fetch_project_req(
    db_pool: web::Data<PgPool>,
    user_account: UserAccount,
    body: ProjectFetchRequest,
) -> Result<web::Json<GenericResponse<ProjectAccount>>, GenericError> {
    let project_account = get_project_account(&db_pool, user_account.id, body.id)
        .await
        .map_err(|e| GenericError::DatabaseError(e.to_string(), e))?
        .ok_or_else(|| {
            GenericError::ValidationError("project account does not exist.".to_string())
        })?;
    Ok(web::Json(GenericResponse::success(
        "Sucessfully fetched project data.",
        project_account,
    )))
}

#[utoipa::path(
    post,
    path = "/project/permission",
    tag = "Permission",
    description = "API for checking the permission of a project.",
    summary = "project Account Permission API",
    request_body(content = ProjectPermissionRequest, description = "Request Body"),
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
#[tracing::instrument(err, name = "project Permission", skip(db_pool), fields())]
pub async fn project_permission_validation(
    db_pool: web::Data<PgPool>,
    user_account: UserAccount,
    body: ProjectPermissionRequest,
) -> Result<web::Json<GenericResponse<Vec<String>>>, GenericError> {
    let permission_list = validate_user_project_permission(
        &db_pool,
        user_account.id,
        body.project_id,
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
    path = "/project/list",
    tag = "project Account",
    description = "API for listing all project account associated to a user",
    summary = "project Account List API",
    // request_body(content = ProjectAccountListReq, description = "Request Body"),
    responses(
        (status=200, description= "Sucessfully fetched project data.", body= GenericResponse<Vec<BasicprojectAccount>>),
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
#[tracing::instrument(err, name = "fetch project detail", skip(db_pool), fields())]
pub async fn list_project_req(
    db_pool: web::Data<PgPool>,
    user_account: UserAccount,
) -> Result<web::Json<GenericResponse<Vec<BasicprojectAccount>>>, GenericError> {
    let project_obj = if user_account.user_role != RoleType::Superadmin.to_string() {
        get_basic_project_accounts_by_user_id(user_account.id, &db_pool).await
    } else {
        get_basic_project_accounts(&db_pool).await
    }
    .map_err(|e| GenericError::DatabaseError(e.to_string(), e))?;
    Ok(web::Json(GenericResponse::success(
        "Sucessfully fetched all associated project accounts.",
        project_obj,
    )))
}

#[utoipa::path(
    get,
    path = "/user/assocation",
    tag = "project Account",
    description = "API for association of user with project account",
    summary = "Use project Account Association API",
    // request_body(content = ProjectAccountListReq, description = "Request Body"),
    responses(
        (status=200, description= "Sucessfully fetched project data.", body= GenericResponse<TupleUnit>),
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
#[tracing::instrument(err, name = "user project association", skip(db_pool), fields())]
pub async fn user_project_association_req(
    req: ProjectUserAssociationRequest,
    db_pool: web::Data<PgPool>,
    user_account: UserAccount,
    project_account: ProjectAccount,
    websocket_srv: web::Data<Addr<Server>>,
) -> Result<web::Json<GenericResponse<()>>, GenericError> {
    if req.role == RoleType::Superadmin {
        return Err(GenericError::InsufficientPrevilegeError(
            "Insufficient previlege to assign Superadmin".to_string(),
        ));
    }
    let role_obj_task = get_role(&db_pool, &req.role);

    let assocated_user_task = get_project_account(&db_pool, user_account.id, project_account.id);

    let (role_obj_res, assocated_user_res) = tokio::join!(role_obj_task, assocated_user_task);

    let assocated_user = assocated_user_res.map_err(|e| {
        GenericError::DatabaseError(
            "Something went wrong while fetching existing user-project association".to_owned(),
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
            "User already associated with project".to_owned(),
        ));
    }
    associate_user_to_project(
        &db_pool,
        req.user_id,
        project_account.id,
        role_obj.id,
        user_account.id,
    )
    .await
    .map_err(|_| {
        GenericError::UnexpectedCustomError(
            "Something went wrong while associating user to project".to_owned(),
        )
    })?;
    let msg: MessageToClient = MessageToClient::new(
        WebSocketActionType::UserProjectAssociation,
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
        "Sucessfully associated user with project account.",
        (),
    )))
}
