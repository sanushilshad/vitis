use actix_http::Payload;
use actix_web::{FromRequest, HttpMessage, HttpRequest, web};
use futures::future::LocalBoxFuture;
use serde::{Deserialize, Serialize};
use std::future::{Ready, ready};
use utoipa::ToSchema;
use uuid::Uuid;

use crate::errors::GenericError;
use crate::routes::user::schemas::UserRoleType;
use crate::schemas::Status;
use anyhow::anyhow;

#[allow(dead_code)]
#[derive(Deserialize, Debug, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct CreateDepartmentAccount {
    pub name: String,
    pub is_test_account: bool,
    pub international_dialing_code: String,
}

impl FromRequest for CreateDepartmentAccount {
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
pub struct DepartmentFetchRequest {
    pub id: Uuid,
}

impl FromRequest for DepartmentFetchRequest {
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
pub struct DepartmentPermissionRequest {
    pub action_list: Vec<String>,
    pub department_id: Uuid,
}

impl FromRequest for DepartmentPermissionRequest {
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
pub struct BasicDepartmentAccount {
    pub name: String,
    pub id: Uuid,
}

#[derive(Debug, Deserialize, Clone, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct DepartmentAccount {
    pub id: Uuid,
    pub name: String,
    pub is_active: Status,
    pub is_deleted: bool,
    pub verified: bool,
}

impl FromRequest for DepartmentAccount {
    type Error = GenericError;
    type Future = Ready<Result<Self, Self::Error>>;

    fn from_request(
        req: &actix_web::HttpRequest,
        _payload: &mut actix_web::dev::Payload,
    ) -> Self::Future {
        let value = req.extensions().get::<DepartmentAccount>().cloned();

        let result = match value {
            Some(user) => Ok(user),
            None => Err(GenericError::UnexpectedError(anyhow!(
                "Something went wrong while parsing department Account data".to_string()
            ))),
        };

        ready(result)
    }
}

// #[derive(Debug, Serialize, ToSchema)]
// pub struct WSdepartmentAccountCreate {
//     pub message: String,
// }

// impl WSdepartmentAccountCreate {
//     pub fn get_message(message: String) -> Self {
//         Self { message }
//     }
// }

// #[derive(Debug, Deserialize, ToSchema)]
// pub struct departmentAccountListReq {}

// impl FromRequest for departmentAccountListReq {
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
pub struct DepartmentUserAssociationRequest {
    pub user_id: Uuid,
    pub department_id: Uuid,
    pub role: UserRoleType,
}

impl FromRequest for DepartmentUserAssociationRequest {
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
