use std::collections::{HashMap, HashSet};

use actix_web::web;
use sqlx::PgPool;
use utoipa::TupleUnit;

use crate::{
    errors::GenericError,
    routes::{business::schemas::BusinessAccount, user::schemas::UserAccount},
    schemas::{AllowedPermission, GenericResponse, PermissionType},
};

use super::{
    models::SettingModel,
    schemas::{
        CreateBusinessSettingRequest, CreateGlobalSettingRequest, CreateUserSettingRequest,
        FetchSettingEnumRequest, FetchSettingRequest, SettingData, SettingEnumData, SettingType,
    },
    utils::{
        // create_business_setting, create_global_setting, create_user_business_setting,
        // create_user_setting,
        create_setting_with_scope,
        fetch_setting,
        fetch_setting_enums,
        get_setting_value,
    },
};

#[utoipa::path(
    post,
    description = "API for creating configs specific to business.",
    summary = "Business Setting Create API",
    path = "/setting/business/save",
    tag = "Setting",
    request_body(content = CreateBusinessSettingRequest, description = "Request Body"),
    responses(
        (status=200, description= "business Account created successfully", body= GenericResponse<TupleUnit>),
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
#[tracing::instrument(err, name = "Business Config Creation API", skip(pool, body), fields())]
pub async fn create_business_config_req(
    body: CreateBusinessSettingRequest,
    pool: web::Data<PgPool>,
    user: UserAccount,
    business_account: BusinessAccount,
) -> Result<web::Json<GenericResponse<()>>, GenericError> {
    let key_list: Vec<String> = body.settings.iter().map(|a| a.key.to_owned()).collect();
    let valid_settings = fetch_setting(&pool, &key_list, SettingType::Business)
        .await
        .map_err(|e| GenericError::DatabaseError(e.to_string(), e))?;
    let setting_map: HashMap<String, &SettingModel> = valid_settings
        .iter()
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
    create_setting_with_scope(
        &pool,
        &body.settings,
        None,
        Some(business_account.id),
        user.id,
        &setting_map,
    )
    .await
    .map_err(|e| GenericError::DatabaseError(e.to_string(), e))?;
    Ok(web::Json(GenericResponse::success(
        "Sucessfully created Business config/s",
        (),
    )))
}

#[utoipa::path(
    post,
    description = "API for creating configs specific to user.",
    summary = "User Setting Create API",
    path = "/setting/user/save",
    tag = "Setting",
    request_body(content = CreateUserSettingRequest, description = "Request Body"),
    responses(
        (status=200, description= "business Account created successfully", body= GenericResponse<TupleUnit>),
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
#[tracing::instrument(err, name = "User Config Creation API", skip(pool, body), fields())]
pub async fn create_user_config_req(
    body: CreateUserSettingRequest,
    pool: web::Data<PgPool>,
    user: UserAccount,
    permissions: AllowedPermission,
) -> Result<web::Json<GenericResponse<()>>, GenericError> {
    let key_list: Vec<String> = body.settings.iter().map(|a| a.key.to_owned()).collect();
    let valid_settings = fetch_setting(&pool, &key_list, SettingType::User)
        .await
        .map_err(|e| GenericError::DatabaseError(e.to_string(), e))?;
    let mut setting_map = HashMap::new();
    let mut restricted_keys = Vec::new();
    let mut found_keys = HashSet::new();
    let has_create_permission = permissions
        .permission_list
        .contains(&PermissionType::CreateUserSetting.to_string());
    for setting in valid_settings.iter() {
        found_keys.insert(&setting.key);

        if !setting.is_editable {
            restricted_keys.push(setting.key.clone());
        }
        setting_map.insert(setting.key.clone(), setting);
    }

    if !has_create_permission && !restricted_keys.is_empty() {
        let restricted_keys_str = restricted_keys.join(", ");
        return Err(GenericError::ValidationError(format!(
            "Restricted key/s cannot be modified: {}",
            restricted_keys_str
        )));
    }

    let user_id = if has_create_permission {
        body.user_id.unwrap_or(user.id)
    } else {
        user.id
    };

    let invalid_keys: Vec<&String> = key_list
        .iter()
        .filter(|key| !found_keys.contains(*key))
        .collect();

    if !invalid_keys.is_empty() {
        let invalid_keys_str = invalid_keys
            .iter()
            .map(|s| s.as_str())
            .collect::<Vec<&str>>()
            .join(", ");
        return Err(GenericError::ValidationError(format!(
            "Invalid key/s not found in DB: {}",
            invalid_keys_str
        )));
    }
    create_setting_with_scope(
        &pool,
        &body.settings,
        Some(user_id),
        None,
        user.id,
        &setting_map,
    )
    .await
    .map_err(|e| GenericError::DatabaseError(e.to_string(), e))?;
    Ok(web::Json(GenericResponse::success(
        "Sucessfully created User config/s",
        (),
    )))
}

#[utoipa::path(
    post,
    description = "API for fetching configs specific to user/business/TSP.",
    summary = "Business Setting Fetch API",
    path = "/setting/business/fetch",
    tag = "Setting",
    request_body(content = FetchSettingRequest, description = "Request Body"),
    responses(
        (status=200, description= "business Account created successfully", body= GenericResponse<SettingData>),
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
#[tracing::instrument(err, name = "Business Config Fetch API", skip(pool, body), fields())]
pub async fn fetch_business_config_req(
    body: FetchSettingRequest,
    pool: web::Data<PgPool>,
    user: UserAccount,
    business_account: BusinessAccount,
) -> Result<web::Json<GenericResponse<SettingData>>, GenericError> {
    let settings = get_setting_value(&pool, &body.keys, Some(business_account.id), None, false)
        .await
        .map_err(|e| GenericError::DatabaseError(e.to_string(), e))?;
    let data = SettingData { settings };
    Ok(web::Json(GenericResponse::success(
        "Sucessfully fetched business config/s",
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
        (status=200, description= "business Account created successfully", body= GenericResponse<SettingData>),
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
#[tracing::instrument(err, name = "User Config Fetch API", skip(pool, body), fields())]
pub async fn fetch_user_config_req(
    body: FetchSettingRequest,
    pool: web::Data<PgPool>,
    user: UserAccount,
    permissions: AllowedPermission,
) -> Result<web::Json<GenericResponse<SettingData>>, GenericError> {
    let user_id = if body.user_id.is_some()
        && permissions
            .permission_list
            .contains(&PermissionType::CreateUserSetting.to_string())
    {
        body.user_id
    } else {
        Some(user.id)
    };
    let settings = get_setting_value(&pool, &body.keys, None, user_id, true)
        .await
        .map_err(|e| GenericError::DatabaseError(e.to_string(), e))?;
    let data = SettingData { settings };
    Ok(web::Json(GenericResponse::success(
        "Sucessfully fetched user config/s",
        data,
    )))
}

#[utoipa::path(
    post,
    description = "API for fetching global configs.",
    summary = "Global Setting Fetch API",
    path = "/setting/global/fetch",
    tag = "Setting",
    request_body(content = FetchSettingRequest, description = "Request Body"),
    responses(
        (status=200, description= "business Account created successfully", body= GenericResponse<SettingData>),
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
#[tracing::instrument(err, name = "Global Config Fetch API", skip(pool, body), fields())]
pub async fn fetch_global_setting(
    body: FetchSettingRequest,
    pool: web::Data<PgPool>,
    user: UserAccount,
) -> Result<web::Json<GenericResponse<SettingData>>, GenericError> {
    let settings = get_setting_value(&pool, &body.keys, None, None, false)
        .await
        .map_err(|e| GenericError::DatabaseError(e.to_string(), e))?;
    let data = SettingData { settings };
    Ok(web::Json(GenericResponse::success(
        "Sucessfully fetched allowed config/s",
        data,
    )))
}

#[utoipa::path(
    post,
    description = "API for creating global configs.",
    summary = "Global Setting Create API",
    path = "/setting/global/save",
    tag = "Setting",
    request_body(content = CreateGlobalSettingRequest, description = "Request Body"),
    responses(
        (status=200, description= "business Account created successfully", body= GenericResponse<TupleUnit>),
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
#[tracing::instrument(err, name = "Global Config Creation API", skip(pool, body), fields())]
pub async fn save_global_setting(
    body: CreateGlobalSettingRequest,
    pool: web::Data<PgPool>,
    user: UserAccount,
) -> Result<web::Json<GenericResponse<()>>, GenericError> {
    let key_list: Vec<String> = body.settings.iter().map(|a| a.key.to_owned()).collect();
    let valid_settings = fetch_setting(&pool, &key_list, SettingType::Global)
        .await
        .map_err(|e| GenericError::DatabaseError(e.to_string(), e))?;
    let mut setting_map = HashMap::new();
    let mut restricted_keys = Vec::new();
    let mut found_keys = HashSet::new();
    for setting in valid_settings.iter() {
        found_keys.insert(&setting.key);

        if !setting.is_editable {
            restricted_keys.push(setting.key.clone());
        }
        setting_map.insert(setting.key.clone(), setting);
    }

    let invalid_keys: Vec<&String> = key_list
        .iter()
        .filter(|key| !found_keys.contains(*key))
        .collect();

    if !invalid_keys.is_empty() {
        let invalid_keys_str = invalid_keys
            .iter()
            .map(|s| s.as_str())
            .collect::<Vec<&str>>()
            .join(", ");
        return Err(GenericError::ValidationError(format!(
            "Invalid key/s not found in DB: {}",
            invalid_keys_str
        )));
    }
    create_setting_with_scope(&pool, &body.settings, None, None, user.id, &setting_map)
        .await
        .map_err(|e| GenericError::DatabaseError(e.to_string(), e))?;
    Ok(web::Json(GenericResponse::success(
        "Sucessfully created Global config/s",
        (),
    )))
}

#[utoipa::path(
    post,
    description = "API for fetching setting enums",
    summary = "Setting Enum Fetch API",
    path = "/setting/enum/fetch",
    tag = "Setting",
    request_body(content = FetchSettingEnumRequest, description = "Request Body"),
    responses(
        (status=200, description= "business Account created successfully", body= GenericResponse<Vec<SettingEnumData>>),
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
#[tracing::instrument(err, name = "Global Config Fetch API", skip(pool, body), fields())]
pub async fn fetch_config_enums(
    body: FetchSettingEnumRequest,
    pool: web::Data<PgPool>,
    user: UserAccount,
) -> Result<web::Json<GenericResponse<Vec<SettingEnumData>>>, GenericError> {
    let data = fetch_setting_enums(&pool, &body.id_list)
        .await
        .map_err(|e| GenericError::DatabaseError("Something went fetching enums".to_string(), e))?;
    Ok(web::Json(GenericResponse::success(
        "Sucessfully fetched allowed config enums",
        data,
    )))
}

#[utoipa::path(
    post,
    description = "API for fetching configs specific to user-business.",
    summary = "Business Setting Fetch API",
    path = "/setting/user-business/fetch",
    tag = "Setting",
    request_body(content = FetchSettingRequest, description = "Request Body"),
    responses(
        (status=200, description= "business Account created successfully", body= GenericResponse<SettingData>),
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
#[tracing::instrument(
    err,
    name = "User-Business Config Fetch API",
    skip(pool, body),
    fields()
)]
pub async fn fetch_user_business_config_req(
    body: FetchSettingRequest,
    pool: web::Data<PgPool>,
    user: UserAccount,
    business_account: BusinessAccount,
) -> Result<web::Json<GenericResponse<SettingData>>, GenericError> {
    let settings = get_setting_value(
        &pool,
        &body.keys,
        Some(business_account.id),
        Some(user.id),
        false,
    )
    .await
    .map_err(|e| GenericError::DatabaseError(e.to_string(), e))?;
    let data = SettingData { settings };
    Ok(web::Json(GenericResponse::success(
        "Sucessfully fetched business config/s",
        data,
    )))
}

#[utoipa::path(
    post,
    description = "API for creating configs specific to user-business.",
    summary = "User Business Setting Create API",
    path = "/setting/user-business/save",
    tag = "Setting",
    request_body(content = CreateBusinessSettingRequest, description = "Request Body"),
    responses(
        (status=200, description= "business Account created successfully", body= GenericResponse<TupleUnit>),
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
#[tracing::instrument(
    err,
    name = "User-Business Config Creation API",
    skip(pool, body),
    fields()
)]
pub async fn create_user_business_config_req(
    body: CreateBusinessSettingRequest,
    pool: web::Data<PgPool>,
    user: UserAccount,
    business_account: BusinessAccount,
) -> Result<web::Json<GenericResponse<()>>, GenericError> {
    let key_list: Vec<String> = body.settings.iter().map(|a| a.key.to_owned()).collect();
    let valid_settings = fetch_setting(&pool, &key_list, SettingType::UserBusiness)
        .await
        .map_err(|e| GenericError::DatabaseError(e.to_string(), e))?;
    let setting_map: HashMap<String, &SettingModel> = valid_settings
        .iter()
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
    create_setting_with_scope(
        &pool,
        &body.settings,
        Some(user.id),
        Some(business_account.id),
        user.id,
        &setting_map,
    )
    .await
    .map_err(|e| GenericError::DatabaseError(e.to_string(), e))?;
    Ok(web::Json(GenericResponse::success(
        "Sucessfully created User-Business config/s",
        (),
    )))
}
