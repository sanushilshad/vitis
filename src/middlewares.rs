use crate::configuration::SecretConfig;
use crate::errors::GenericError;
use crate::routes::project::schemas::{AllowedPermission, ProjectAccount};
use crate::routes::project::utils::{
    get_project_account, validate_project_account_active, validate_user_project_permission,
};
use crate::routes::user::schemas::UserAccount;
use crate::routes::user::utils::get_user;
use crate::schemas::{RequestMetaData, Status};
use crate::utils::{decode_token, get_header_value};

use actix_http::header::UPGRADE;
use actix_http::{Payload, h1};
use actix_web::body::{self, BoxBody};
use actix_web::dev::{Service, ServiceRequest, ServiceResponse, Transform, forward_ready};
use actix_web::{Error, HttpMessage, http, web};
use futures::future::LocalBoxFuture;
use sqlx::PgPool;
use std::cell::RefCell;
use std::future::{Ready, ready};
use std::rc::Rc;
use tracing::instrument;
use uuid::Uuid;

pub struct AuthMiddleware<S> {
    service: Rc<S>,
}

impl<S> Service<ServiceRequest> for AuthMiddleware<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<actix_web::body::BoxBody>, Error = Error>
        + 'static,
{
    type Response = ServiceResponse<actix_web::body::BoxBody>;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Error>>;

    forward_ready!(service);

    #[instrument(skip(self), name = "AuthMiddleware", fields(path = %req.path()))]
    fn call(&self, req: ServiceRequest) -> Self::Future {
        let token = req
            .cookie("token")
            .map(|c| c.value().to_string())
            .or_else(|| {
                req.headers()
                    .get(http::header::AUTHORIZATION)
                    .map(|h| h.to_str().unwrap().split_at(7).1.to_string())
            });

        let jwt_secret = &req
            .app_data::<web::Data<SecretConfig>>()
            .unwrap()
            .jwt
            .secret;

        if token.is_none() {
            let error_message = "Authorization header is missing".to_string();
            let (request, _pl) = req.into_parts();
            let json_error = GenericError::ValidationError(error_message);
            return Box::pin(async { Ok(ServiceResponse::from_err(json_error, request)) });
        }

        let decoded_user_id = match decode_token(token.unwrap(), jwt_secret) {
            Ok(id) => id,
            Err(e) => {
                return Box::pin(async move {
                    Ok(ServiceResponse::from_err(
                        GenericError::UnAuthorized(e.to_string()),
                        req.into_parts().0,
                    ))
                });
            }
        };

        let srv = Rc::clone(&self.service);
        Box::pin(async move {
            let db_pool = &req.app_data::<web::Data<PgPool>>().unwrap();
            let user = get_user(vec![&decoded_user_id.to_string()], db_pool)
                .await
                .map_err(GenericError::UnexpectedError)?;
            if user.is_active == Status::Inactive {
                return Err(GenericError::ValidationError(
                    "User is Inactive. Please contact customer support".to_string(),
                ))?;
            } else if user.is_deleted {
                return Err(GenericError::ValidationError(
                    "User is in deleted. Please contact customer support".to_string(),
                ))?;
            }
            req.extensions_mut().insert::<UserAccount>(user);

            let res = srv.call(req).await?;
            Ok(res)
        })
    }
}

/// Middleware factory for requiring authentication.
pub struct RequireAuth;

impl<S> Transform<S, ServiceRequest> for RequireAuth
where
    S: Service<ServiceRequest, Response = ServiceResponse<actix_web::body::BoxBody>, Error = Error>
        + 'static,
{
    type Response = ServiceResponse<actix_web::body::BoxBody>;
    type Error = Error;
    type Transform = AuthMiddleware<S>;
    type InitError = ();
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(AuthMiddleware {
            service: Rc::new(service),
        }))
    }
}

//Middleware to validate the project account
pub struct ProjectAccountMiddleware<S> {
    service: Rc<S>,
}
impl<S> Service<ServiceRequest> for ProjectAccountMiddleware<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<actix_web::body::BoxBody>, Error = Error>
        + 'static,
{
    type Response = ServiceResponse<actix_web::body::BoxBody>;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Error>>;

    forward_ready!(service);

    /// Handles incoming requests.
    #[instrument(skip(self), name = "project Account Middleware", fields(path = %req.path()))]
    fn call(&self, req: ServiceRequest) -> Self::Future {
        let srv = Rc::clone(&self.service);

        Box::pin(async move {
            let db_pool = req.app_data::<web::Data<PgPool>>().unwrap();
            let user_account = req
                .extensions()
                .get::<UserAccount>()
                .ok_or_else(|| {
                    GenericError::ValidationError("User Account doesn't exist".to_string())
                })?
                .to_owned();

            if let Some(project_id) = get_header_value(&req, "x-project-id") // Convert HeaderValue to &str
                .and_then(|value| Uuid::parse_str(value).ok())
            {
                let project_account = get_project_account(db_pool, user_account.id, project_id)
                    .await
                    .map_err(GenericError::UnexpectedError)?;
                let extracted_project_account = project_account.ok_or_else(|| {
                    GenericError::ValidationError("project Account doesn't exist".to_string())
                })?;
                let error_message = validate_project_account_active(&extracted_project_account);
                if let Some(message) = error_message {
                    let (request, _pl) = req.into_parts();
                    let json_error = GenericError::ValidationError(message);
                    return Ok(ServiceResponse::from_err(json_error, request));
                }
                req.extensions_mut()
                    .insert::<ProjectAccount>(extracted_project_account);
            } else {
                let (request, _pl) = req.into_parts();
                return Ok(ServiceResponse::from_err(
                    GenericError::ValidationError("Please set x-project-id".to_string()),
                    request,
                ));
            }

            let res = srv.call(req).await?;
            Ok(res)
        })
    }
}

pub struct ProjectAccountValidation {}

impl<S> Transform<S, ServiceRequest> for ProjectAccountValidation
where
    S: Service<ServiceRequest, Response = ServiceResponse<actix_web::body::BoxBody>, Error = Error>
        + 'static,
{
    type Response = ServiceResponse<actix_web::body::BoxBody>;
    type Error = Error;
    type Transform = ProjectAccountMiddleware<S>;
    type InitError = ();
    type Future = Ready<Result<Self::Transform, Self::InitError>>;
    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(ProjectAccountMiddleware {
            service: Rc::new(service),
        }))
    }
}

pub struct HeaderMiddleware<S> {
    service: Rc<S>,
}

impl<S> Service<ServiceRequest> for HeaderMiddleware<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<actix_web::body::BoxBody>, Error = Error>
        + 'static,
{
    type Response = ServiceResponse<actix_web::body::BoxBody>;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Error>>;

    forward_ready!(service);

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let request_id = get_header_value(&req, "x-request-id");
        let device_id = get_header_value(&req, "x-device-id");
        // let _hostname = get_header_value(&req, "Host");

        if request_id.is_none() || device_id.is_none() {
            let error_message = match (request_id.is_none(), device_id.is_none()) {
                (true, _) => "x-request-id is missing".to_string(),
                (_, true) => "x-device-id is missing".to_string(),
                _ => "".to_string(), // Default case, if none of the conditions are met
            };
            let (request, _pl) = req.into_parts();
            let json_error: GenericError = GenericError::ValidationError(error_message);
            return Box::pin(async { Ok(ServiceResponse::from_err(json_error, request)) });
        } else {
            let meta_data = RequestMetaData {
                request_id: request_id.unwrap().to_owned(),
                device_id: device_id.unwrap().to_owned(),
            };
            req.extensions_mut().insert::<RequestMetaData>(meta_data);
        }

        let srv = Rc::clone(&self.service);
        Box::pin(async move {
            let res = srv.call(req).await?;
            Ok(res)
        })
    }
}

/// Middleware factory for requiring authentication.
pub struct HeaderValidation;

impl<S> Transform<S, ServiceRequest> for HeaderValidation
where
    S: Service<ServiceRequest, Response = ServiceResponse<actix_web::body::BoxBody>, Error = Error>
        + 'static,
{
    type Response = ServiceResponse<actix_web::body::BoxBody>;
    type Error = Error;
    type Transform = HeaderMiddleware<S>;
    type InitError = ();
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    /// Creates and returns a new AuthMiddleware wrapped in a Result.
    fn new_transform(&self, service: S) -> Self::Future {
        // Wrap the AuthMiddleware instance in a Result and return it.
        ready(Ok(HeaderMiddleware {
            service: Rc::new(service),
        }))
    }
}

pub fn bytes_to_payload(buf: web::Bytes) -> Payload {
    let (_, mut pl) = h1::Payload::create(true);
    pl.unread_data(buf);
    Payload::from(pl)
}

pub struct ReadReqResMiddleware<S> {
    service: Rc<RefCell<S>>,
}

impl<S> Service<ServiceRequest> for ReadReqResMiddleware<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<actix_web::body::BoxBody>, Error = Error>
        + 'static,
{
    type Response = ServiceResponse<actix_web::body::BoxBody>;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Error>>;

    forward_ready!(service);

    #[instrument(skip(self), name = "Request Response Payload", fields(path = %req.path()))]
    fn call(&self, mut req: ServiceRequest) -> Self::Future {
        let svc = self.service.clone();
        //
        let is_websocket = req.headers().contains_key(UPGRADE)
            && req.headers().get(UPGRADE).unwrap() == "websocket";
        let is_on_search = req.path().ends_with("on_search");
        let is_non_json_req_res =
            req.path().contains("/docs/") || req.path().contains("/api-docs/");
        if is_websocket || is_non_json_req_res {
            Box::pin(async move {
                let fut = svc.call(req).await?;
                Ok(fut)
            })
        } else {
            Box::pin(async move {
                if !is_on_search {
                    let request_str: String = req.extract::<String>().await?;
                    tracing::info!({%request_str}, "HTTP Response");
                    req.set_payload(bytes_to_payload(web::Bytes::from(request_str)));
                }
                let fut = svc.call(req).await?;

                let (req, res) = fut.into_parts();
                let (res, body) = res.into_parts();
                let body_bytes = body::to_bytes(body).await.ok().unwrap();
                let response_str = match std::str::from_utf8(&body_bytes) {
                    Ok(s) => s.to_string(),
                    Err(_) => {
                        tracing::error!("Error decoding response body");
                        String::from("")
                    }
                };
                tracing::info!({%response_str}, "HTTP Response");
                let res = res.set_body(BoxBody::new(response_str));
                let res = ServiceResponse::new(req, res);
                Ok(res)
            })
        }
    }
}

pub struct SaveRequestResponse;

impl<S> Transform<S, ServiceRequest> for SaveRequestResponse
where
    S: Service<ServiceRequest, Response = ServiceResponse<actix_web::body::BoxBody>, Error = Error>
        + 'static,
{
    type Response = ServiceResponse<actix_web::body::BoxBody>;
    type Error = Error;
    type InitError = ();
    type Transform = ReadReqResMiddleware<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(ReadReqResMiddleware {
            service: Rc::new(RefCell::new(service)),
        }))
    }
}

pub struct ProjectPermissionMiddleware<S> {
    service: Rc<S>,
    pub permission_list: Vec<String>,
}
impl<S> Service<ServiceRequest> for ProjectPermissionMiddleware<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<actix_web::body::BoxBody>, Error = Error>
        + 'static,
{
    type Response = ServiceResponse<actix_web::body::BoxBody>;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Error>>;

    forward_ready!(service);

    /// Handles incoming requests.
    fn call(&self, req: ServiceRequest) -> Self::Future {
        let srv = Rc::clone(&self.service);

        let permission_list = self.permission_list.clone();

        Box::pin(async move {
            let db_pool = req.app_data::<web::Data<PgPool>>().unwrap();
            let user_account = req
                .extensions()
                .get::<UserAccount>()
                .ok_or_else(|| {
                    GenericError::ValidationError("User Account doesn't exist".to_string())
                })?
                .to_owned();
            let project_account = req
                .extensions()
                .get::<ProjectAccount>()
                .ok_or_else(|| {
                    GenericError::ValidationError(
                        "project Account Account doesn't exist".to_string(),
                    )
                })?
                .to_owned();
            let permission_list = validate_user_project_permission(
                db_pool,
                user_account.id,
                project_account.id,
                &permission_list,
            )
            .await
            .map_err(|e| {
                GenericError::DatabaseError(
                    "Something went wrong while fetching permission".to_owned(),
                    e,
                )
            })?;
            if permission_list.is_empty() {
                let (request, _pl) = req.into_parts();
                return Ok(ServiceResponse::from_err(
                    GenericError::InsufficientPrevilegeError(
                        "User doesn't have sufficient permission for the given action".to_owned(),
                    ),
                    request,
                ));
            }

            req.extensions_mut()
                .insert::<AllowedPermission>(AllowedPermission { permission_list });

            let res = srv.call(req).await?;
            Ok(res)
        })
    }
}

// Middleware factory for project account validation.
pub struct ProjectPermissionValidation {
    pub permission_list: Vec<String>,
}

impl<S> Transform<S, ServiceRequest> for ProjectPermissionValidation
where
    S: Service<ServiceRequest, Response = ServiceResponse<actix_web::body::BoxBody>, Error = Error>
        + 'static,
{
    type Response = ServiceResponse<actix_web::body::BoxBody>;
    type Error = Error;
    type Transform = ProjectPermissionMiddleware<S>;
    type InitError = ();
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    /// Creates and returns a new AuthMiddleware wrapped in a Result.
    fn new_transform(&self, service: S) -> Self::Future {
        // Wrap the AuthMiddleware instance in a Result and return it.
        ready(Ok(ProjectPermissionMiddleware {
            service: Rc::new(service),
            permission_list: self.permission_list.clone(),
        }))
    }
}
