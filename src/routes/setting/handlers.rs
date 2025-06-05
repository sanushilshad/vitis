use std::collections::{HashMap, HashSet};

use actix_web::web;
use sqlx::PgPool;
use utoipa::TupleUnit;

use crate::{
    errors::GenericError, routes::project::schemas::ProjectAccount,
    routes::user::schemas::UserAccount, schemas::GenericResponse,
};

use super::{
    models::SettingModel,
    schemas::{
        CreateProjectSettingRequest, CreateUserSettingRequest, FetchSettingRequest, SettingData,
        SettingType,
    },
    utils::{create_project_setting, create_user_setting, fetch_setting, get_setting_value},
};

#[utoipa::path(
    post,
    description = "API for creating configs specific to project.",
    summary = "Project Setting Create API",
    path = "/setting/project/create",
    tag = "Setting",
    request_body(content = CreateProjectSettingRequest, description = "Request Body"),
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
#[tracing::instrument(err, name = "Project Config Creation API", skip(pool, body), fields())]
pub async fn create_project_config_req(
    body: CreateProjectSettingRequest,
    pool: web::Data<PgPool>,
    user: UserAccount,
    project_account: ProjectAccount,
) -> Result<web::Json<GenericResponse<()>>, GenericError> {
    let key_list: Vec<String> = body.settings.iter().map(|a| a.key.to_owned()).collect();
    let valid_settings = fetch_setting(&pool, &key_list, SettingType::Project)
        .await
        .map_err(|e| GenericError::DatabaseError(e.to_string(), e))?;
    let setting_map: HashMap<String, SettingModel> = valid_settings
        .into_iter()
        .filter(|e| e.is_editable)
        .map(|setting| (setting.key.to_owned(), setting))
        .collect();
    if setting_map.len() != key_list.len() {
        let valid_keys_set: HashSet<&String> = setting_map.iter().map(|e| &e.1.key).collect();
        let invalid_keys: Vec<&String> = key_list
            .iter()
            .filter(|key| !valid_keys_set.contains(key))
            .collect();
        let invalid_keys_str = invalid_keys
            .iter()
            .map(|s| s.as_str())
            .collect::<Vec<&str>>()
            .join(", ");
        return Err(GenericError::ValidationError(format!(
            "Invalid Key/s: {}",
            invalid_keys_str
        )));
    }
    create_project_setting(&pool, &body, user.id, project_account.id, &setting_map)
        .await
        .map_err(|e| GenericError::DatabaseError(e.to_string(), e))?;
    Ok(web::Json(GenericResponse::success(
        "Sucessfully created Project config/s",
        (),
    )))
}

#[utoipa::path(
    post,
    description = "API for creating configs specific to user.",
    summary = "User Setting Create API",
    path = "/setting/user/create",
    tag = "Setting",
    request_body(content = CreateProjectSettingRequest, description = "Request Body"),
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
#[tracing::instrument(err, name = "User Config Creation API", skip(pool, body), fields())]
pub async fn create_user_config_req(
    body: CreateUserSettingRequest,
    pool: web::Data<PgPool>,
    user: UserAccount,
) -> Result<web::Json<GenericResponse<()>>, GenericError> {
    let key_list: Vec<String> = body.settings.iter().map(|a| a.key.to_owned()).collect();
    let valid_settings = fetch_setting(&pool, &key_list, SettingType::User)
        .await
        .map_err(|e| GenericError::DatabaseError(e.to_string(), e))?;
    let setting_map: HashMap<String, SettingModel> = valid_settings
        .into_iter()
        .filter(|e| e.is_editable)
        .map(|setting| (setting.key.to_owned(), setting))
        .collect();
    if setting_map.len() != key_list.len() {
        let valid_keys_set: HashSet<&String> = setting_map.iter().map(|e| &e.1.key).collect();
        let invalid_keys: Vec<&String> = key_list
            .iter()
            .filter(|key| !valid_keys_set.contains(key))
            .collect();
        let invalid_keys_str = invalid_keys
            .iter()
            .map(|s| s.as_str())
            .collect::<Vec<&str>>()
            .join(", ");
        return Err(GenericError::ValidationError(format!(
            "Invalid Key/s: {}",
            invalid_keys_str
        )));
    }
    create_user_setting(&pool, &body, user.id, &setting_map)
        .await
        .map_err(|e| GenericError::DatabaseError(e.to_string(), e))?;
    Ok(web::Json(GenericResponse::success(
        "Sucessfully created User config/s",
        (),
    )))
}

#[utoipa::path(
    post,
    description = "API for fetching configs specific to user/project/TSP.",
    summary = "Project Setting Fetch API",
    path = "/setting/project/fetch",
    tag = "Setting",
    request_body(content = FetchSettingRequest, description = "Request Body"),
    responses(
        (status=200, description= "project Account created successfully", body= GenericResponse<SettingData>),
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
#[tracing::instrument(err, name = "Project Config Fetch API", skip(pool, body), fields())]
pub async fn fetch_project_config_req(
    body: FetchSettingRequest,
    pool: web::Data<PgPool>,
    user: UserAccount,
    project_account: ProjectAccount,
) -> Result<web::Json<GenericResponse<SettingData>>, GenericError> {
    let settings = get_setting_value(&pool, &body.keys, Some(project_account.id), user.id)
        .await
        .map_err(|e| GenericError::DatabaseError(e.to_string(), e))?;
    let data = SettingData { settings };
    Ok(web::Json(GenericResponse::success(
        "Sucessfully fetched project config/s",
        data,
    )))
}

#[utoipa::path(
    post,
    description = "API for fetching configs specific to user.",
    summary = "User Setting Fetch API",
    path = "/setting/user/fetch",
    tag = "Setting",
    request_body(content = FetchSettingRequest, description = "Request Body"),
    responses(
        (status=200, description= "project Account created successfully", body= GenericResponse<SettingData>),
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
#[tracing::instrument(err, name = "User Config Fetch API", skip(pool, body), fields())]
pub async fn fetch_user_config_req(
    body: FetchSettingRequest,
    pool: web::Data<PgPool>,
    user: UserAccount,
) -> Result<web::Json<GenericResponse<SettingData>>, GenericError> {
    let settings = get_setting_value(&pool, &body.keys, None, user.id)
        .await
        .map_err(|e| GenericError::DatabaseError(e.to_string(), e))?;
    let data = SettingData { settings };
    Ok(web::Json(GenericResponse::success(
        "Sucessfully fetched user config/s",
        data,
    )))
}
