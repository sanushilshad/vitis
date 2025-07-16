use actix_http::Payload;
use actix_web::{FromRequest, HttpRequest, web};
use chrono::{DateTime, Utc};
use futures::future::LocalBoxFuture;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

use crate::errors::GenericError;

#[derive(Debug, ToSchema, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Permission {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
}

#[derive(Deserialize, Debug, ToSchema, PartialEq)]
pub enum PermissionLevel {
    Global,
    User,
    Business,
    Department,
}

#[derive(Debug, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct PermissionRoleAssociationRequest {
    pub permission_id_list: Vec<Uuid>,
    pub role_id: Uuid,
}

impl FromRequest for PermissionRoleAssociationRequest {
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

#[derive(Debug)]
pub struct BulkPermissionInsert {
    pub id: Vec<Uuid>,
    pub role_id: Vec<Uuid>,
    pub permission_id: Vec<Uuid>,
    pub created_on: Vec<DateTime<Utc>>,
    pub created_by: Vec<Uuid>,
}
