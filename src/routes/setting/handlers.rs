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
    schemas::{CreateSettingRequest, FetchSettingRequest, SettingData},
    utils::{create_setting, fetch_setting, get_setting_value},
};

#[utoipa::path(
    post,
    description = "API for creating configs specific to user/project/TSP.",
    summary = "Setting Create API",
    path = "/setting/create",
    tag = "Setting",
    request_body(content = CreateSettingRequest, description = "Request Body"),
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
      )
)]
#[tracing::instrument(err, name = "Config Creation API", skip(pool, body), fields())]
pub async fn create_config_req(
    body: CreateSettingRequest,
    pool: web::Data<PgPool>,
    user: UserAccount,
    project_account: ProjectAccount,
) -> Result<web::Json<GenericResponse<()>>, GenericError> {
    let key_list: Vec<String> = body.settings.iter().map(|a| a.key.to_owned()).collect();
    let valid_settings = fetch_setting(&pool, &key_list)
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
    create_setting(&pool, &body, user.id, project_account.id, &setting_map)
        .await
        .map_err(|e| GenericError::DatabaseError(e.to_string(), e))?;
    Ok(web::Json(GenericResponse::success(
        "Sucessfully created config/s",
        (),
    )))
}

#[utoipa::path(
    post,
    description = "API for fetching configs specific to user/project/TSP.",
    summary = "Setting Fetch API",
    path = "/setting/fetch",
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
      )
)]
#[tracing::instrument(err, name = "Config Creation API", skip(pool, body), fields())]
pub async fn fetch_config_req(
    body: FetchSettingRequest,
    pool: web::Data<PgPool>,
    user: UserAccount,
    project_account: ProjectAccount,
) -> Result<web::Json<GenericResponse<SettingData>>, GenericError> {
    let settings = get_setting_value(&pool, &body.keys, project_account.id, user.id)
        .await
        .map_err(|e| GenericError::DatabaseError(e.to_string(), e))?;
    let data = SettingData { settings };
    Ok(web::Json(GenericResponse::success(
        "Sucessfully fetched config/s",
        data,
    )))
}
