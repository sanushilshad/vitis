use crate::errors::GenericError;
use actix_http::StatusCode;
use actix_web::{FromRequest, HttpMessage};

use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

use std::fmt::Debug;
use std::future::{Ready, ready};
#[derive(Serialize, Debug, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct GenericResponse<D> {
    pub status: bool,
    pub customer_message: String,
    pub code: String,
    pub data: D,
}

impl<D> GenericResponse<D> {
    pub fn success(message: &str, data: D) -> Self {
        Self {
            status: true,
            customer_message: String::from(message),
            code: StatusCode::OK.as_str().to_owned(),
            data,
        }
    }

    pub fn error(message: &str, code: StatusCode, data: D) -> Self {
        Self {
            status: false,
            customer_message: String::from(message),
            code: code.as_str().to_owned(),
            data,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, sqlx::Type, Clone, ToSchema)]
#[sqlx(rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum MaskingType {
    NA,
    Encrypt,
    PartialMask,
    FullMask,
}

#[derive(Serialize, Deserialize, Debug, sqlx::Type, Clone, PartialEq, ToSchema)]
#[sqlx(rename_all = "lowercase", type_name = "status")]
#[serde(rename_all = "lowercase")]
pub enum Status {
    Active,
    Inactive,
    Pending,
    Archived,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct RequestMetaData {
    pub device_id: String,
    pub request_id: String,
}

impl FromRequest for RequestMetaData {
    type Error = GenericError;
    type Future = Ready<Result<Self, Self::Error>>;

    fn from_request(
        req: &actix_web::HttpRequest,
        _payload: &mut actix_web::dev::Payload,
    ) -> Self::Future {
        let value = req.extensions().get::<RequestMetaData>().cloned();

        let result = match value {
            Some(data) => Ok(data),

            None => Err(GenericError::ValidationError(
                "Something went wrong while setting meta data".to_string(),
            )),
        };

        ready(result)
    }
}

#[derive(Deserialize, Debug, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum DeleteType {
    Hard,
    Soft,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct WebSocketParam {
    pub user_id: Option<Uuid>,
    pub business_id: Option<Uuid>,
    pub device_id: Option<String>,
}

#[allow(dead_code)]
pub trait WSKeyTrait {
    fn get_ws_key(&self) -> String;
}

impl WSKeyTrait for WebSocketParam {
    fn get_ws_key(&self) -> String {
        format!(
            "{}#{}#{}",
            self.user_id.map_or("NA".to_string(), |id| id.to_string()),
            self.business_id
                .map_or("NA".to_string(), |id| id.to_string()),
            self.device_id.clone().unwrap_or("NA".to_string())
        )
    }
}
