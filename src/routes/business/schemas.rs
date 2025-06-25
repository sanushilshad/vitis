use actix_http::Payload;
use actix_web::{FromRequest, HttpMessage, HttpRequest, web};
use futures::future::LocalBoxFuture;
use serde::{Deserialize, Serialize};
use std::future::{Ready, ready};
use utoipa::ToSchema;
use uuid::Uuid;

use crate::email::EmailObject;
use crate::errors::GenericError;
use crate::routes::user::schemas::UserVector;
use crate::schemas::Status;
use anyhow::anyhow;

#[allow(dead_code)]
#[derive(Deserialize, Debug, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct CreateBusinessAccount {
    pub name: String,
    pub is_test_account: bool,
    pub mobile_no: String,
    pub email: EmailObject,
    pub international_dialing_code: String,
}

impl CreateBusinessAccount {
    pub fn get_full_mobile_no(&self) -> String {
        format!("{}{}", self.international_dialing_code, self.mobile_no)
    }
}
impl FromRequest for CreateBusinessAccount {
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
#[allow(dead_code)]
pub struct BusinessFetchRequest {
    pub id: Uuid,
}

impl FromRequest for BusinessFetchRequest {
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
pub struct BusinessPermissionRequest {
    pub action_list: Vec<String>,
    pub business_id: Uuid,
}

impl FromRequest for BusinessPermissionRequest {
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
pub struct BasicBusinessAccount {
    pub display_name: String,
    pub id: Uuid,
}

#[derive(Debug, Deserialize, Clone, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct BusinessAccount {
    pub id: Uuid,
    pub display_name: String,
    pub vectors: Vec<UserVector>,
    pub is_active: Status,
    pub is_deleted: bool,
    pub verified: bool,
}

impl FromRequest for BusinessAccount {
    type Error = GenericError;
    type Future = Ready<Result<Self, Self::Error>>;

    fn from_request(
        req: &actix_web::HttpRequest,
        _payload: &mut actix_web::dev::Payload,
    ) -> Self::Future {
        let value = req.extensions().get::<BusinessAccount>().cloned();

        let result = match value {
            Some(user) => Ok(user),
            None => Err(GenericError::UnexpectedError(anyhow!(
                "Something went wrong while parsing Business Account data".to_string()
            ))),
        };

        ready(result)
    }
}

// #[derive(Debug, Serialize, ToSchema)]
// pub struct WSBusinessAccountCreate {
//     pub message: String,
// }

// impl WSBusinessAccountCreate {
//     pub fn get_message(message: String) -> Self {
//         Self { message }
//     }
// }

// #[derive(Debug, Deserialize, ToSchema)]
// pub struct BusinessAccountListReq {}

// impl FromRequest for BusinessAccountListReq {
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
pub struct BusinessUserAssociationRequest {
    pub user_id: Uuid,
    pub role_id: Uuid,
}

impl FromRequest for BusinessUserAssociationRequest {
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
