use actix_http::Payload;
use actix_web::{FromRequest, HttpMessage, HttpRequest, web};
use futures::future::LocalBoxFuture;
use serde::{Deserialize, Serialize};
use std::future::{Ready, ready};
use utoipa::ToSchema;
use uuid::Uuid;

use crate::email::EmailObject;
use crate::errors::GenericError;
use crate::routes::user::schemas::{RoleType, UserVector};
use crate::schemas::Status;
use anyhow::anyhow;

#[allow(dead_code)]
#[derive(Deserialize, Debug, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct CreateprojectAccount {
    pub name: String,
    pub is_test_account: bool,
    pub mobile_no: String,
    pub email: EmailObject,
    pub international_dialing_code: String,
}

impl CreateprojectAccount {
    pub fn get_full_mobile_no(&self) -> String {
        format!("{}{}", self.international_dialing_code, self.mobile_no)
    }
}
impl FromRequest for CreateprojectAccount {
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
pub struct ProjectFetchRequest {
    pub id: Uuid,
}

impl FromRequest for ProjectFetchRequest {
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
pub struct ProjectPermissionRequest {
    pub action_list: Vec<String>,
    pub project_id: Uuid,
}

impl FromRequest for ProjectPermissionRequest {
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

#[derive(Debug, Serialize, ToSchema)]
pub struct BasicprojectAccount {
    pub name: String,
    pub id: Uuid,
}

#[derive(Debug, Deserialize, Clone, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct ProjectAccount {
    pub id: Uuid,
    pub name: String,
    pub vectors: Vec<UserVector>,
    pub is_active: Status,
    pub is_deleted: bool,
    pub verified: bool,
}

impl FromRequest for ProjectAccount {
    type Error = GenericError;
    type Future = Ready<Result<Self, Self::Error>>;

    fn from_request(
        req: &actix_web::HttpRequest,
        _payload: &mut actix_web::dev::Payload,
    ) -> Self::Future {
        let value = req.extensions().get::<ProjectAccount>().cloned();

        let result = match value {
            Some(user) => Ok(user),
            None => Err(GenericError::UnexpectedError(anyhow!(
                "Something went wrong while parsing project Account data".to_string()
            ))),
        };

        ready(result)
    }
}

// #[derive(Debug, Serialize, ToSchema)]
// pub struct WSprojectAccountCreate {
//     pub message: String,
// }

// impl WSprojectAccountCreate {
//     pub fn get_message(message: String) -> Self {
//         Self { message }
//     }
// }

// #[derive(Debug, Deserialize, ToSchema)]
// pub struct ProjectAccountListReq {}

// impl FromRequest for ProjectAccountListReq {
//     type Error = GenericError;
//     type Future = LocalBoxFuture<'static, Result<Self, Self::Error>>;

//     fn from_request(req: &HttpRequest, payload: &mut Payload) -> Self::Future {
//         let fut = web::Json::<Self>::from_request(req, payload);

//         Box::pin(async move {
//             match fut.await {
//                 Ok(json) => Ok(json.into_inner()),
//                 Err(e) => Err(GenericError::ValidationError(e.to_string())),
//             }
//         })
//     }
// }

#[derive(Debug, Deserialize, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct ProjectUserAssociationRequest {
    pub user_id: Uuid,
    pub role: RoleType,
}

impl FromRequest for ProjectUserAssociationRequest {
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
