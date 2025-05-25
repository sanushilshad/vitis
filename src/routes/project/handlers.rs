use actix_web::web;
use sqlx::PgPool;
use utoipa::TupleUnit;

use crate::{
    errors::GenericError,
    routes::user::schemas::UserAccount,
    schemas::{GenericResponse, RequestMetaData},
};

use super::{
    schemas::{
        BasicprojectAccount, CreateprojectAccount, ProjectAccount, ProjectFetchRequest,
        ProjectPermissionRequest,
    },
    utils::{
        create_project_account, get_basic_project_account_by_user_id, get_project_account,
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
        .map_err(GenericError::UnexpectedError)?
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
      )
)]
#[tracing::instrument(err, name = "fetch project detail", skip(db_pool), fields())]
pub async fn list_project_req(
    // req: ProjectAccountListReq,
    db_pool: web::Data<PgPool>,
    user_account: UserAccount,
) -> Result<web::Json<GenericResponse<Vec<BasicprojectAccount>>>, GenericError> {
    let project_obj = get_basic_project_account_by_user_id(user_account.id, &db_pool)
        .await
        .map_err(GenericError::UnexpectedError)?;
    Ok(web::Json(GenericResponse::success(
        "Sucessfully fetched all associated project accounts.",
        project_obj,
    )))
}
