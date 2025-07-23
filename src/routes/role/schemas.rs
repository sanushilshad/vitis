use actix_http::Payload;
use actix_web::{FromRequest, HttpRequest, web};
use chrono::{DateTime, Utc};
use futures::future::LocalBoxFuture;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

use crate::{errors::GenericError, schemas::Status};

#[derive(Deserialize, Debug, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct CreateRoleData {
    pub id: Option<Uuid>,
    pub name: String,
}

#[derive(Deserialize, Debug, ToSchema)]
#[serde(rename_all = "camelCase")]
#[allow(dead_code)]
pub struct CreateBusinessRoleRequest {
    pub data: Vec<CreateRoleData>,
}

impl FromRequest for CreateBusinessRoleRequest {
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

#[derive(Debug, ToSchema, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct AccountRole {
    pub id: Uuid,
    pub name: String,
    pub status: Status,
    pub is_deleted: bool,
    pub is_editable: bool,
}

#[derive(Debug)]
pub struct BulkRoleInsert<'a> {
    pub id: Vec<Uuid>,
    pub name: Vec<&'a str>,
    pub created_on: Vec<DateTime<Utc>>,
    pub created_by: Vec<Uuid>,
    pub business_id: Vec<Option<Uuid>>,
    pub department_id: Vec<Option<Uuid>>,
    pub is_editable: Vec<bool>,
}
