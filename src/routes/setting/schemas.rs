use crate::errors::GenericError;
use actix_http::Payload;
use actix_web::{FromRequest, HttpRequest, web};
use futures::future::LocalBoxFuture;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(Deserialize, Debug, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct CreateSettingData {
    pub key: String,
    pub value: String,
}

#[derive(Deserialize, Debug, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct CreateSettingRequest {
    #[schema(value_type = String)]
    pub user_id: Option<Uuid>,
    pub settings: Vec<CreateSettingData>,
}

impl FromRequest for CreateSettingRequest {
    type Error = GenericError;
    type Future = LocalBoxFuture<'static, Result<Self, Self::Error>>;

    fn from_request(req: &HttpRequest, payload: &mut Payload) -> Self::Future {
        let fut = web::Json::<Self>::from_request(req, payload);

        Box::pin(async move {
            match fut.await {
                Ok(json) => Ok(json.into_inner()),
                Err(e) => Err(GenericError::ValidationError(e.to_string())),
            }
        })
    }
}

#[derive(Deserialize, Debug, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct FetchSettingRequest {
    pub keys: Vec<String>,
}

impl FromRequest for FetchSettingRequest {
    type Error = GenericError;
    type Future = LocalBoxFuture<'static, Result<Self, Self::Error>>;

    fn from_request(req: &HttpRequest, payload: &mut Payload) -> Self::Future {
        let fut = web::Json::<Self>::from_request(req, payload);

        Box::pin(async move {
            match fut.await {
                Ok(json) => Ok(json.into_inner()),
                Err(e) => Err(GenericError::ValidationError(e.to_string())),
            }
        })
    }
}

#[derive(Serialize, Debug, ToSchema)]
pub struct Setting {
    pub value: String,
    #[schema(value_type = String)]
    pub id: Uuid,
}

#[derive(Serialize, Debug, ToSchema)]
pub struct Settings {
    pub key: String,
    pub label: String,
    #[schema(value_type = Option<String>)]
    pub enum_id: Option<Uuid>,
    pub is_editable: bool,
    pub global_level: Vec<Setting>,
    pub user_level: Vec<Setting>,
    pub project_level: Vec<Setting>,
}

#[derive(Serialize, Debug, ToSchema)]
pub struct SettingData {
    pub settings: Vec<Settings>,
}
