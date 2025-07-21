use crate::configuration::SecretConfig;
use crate::errors::GenericError;
use crate::routes::business::schemas::BusinessAccount;
use crate::routes::business::utils::{
    fetch_business_account_model_by_id, get_business_account, validate_business_account_active,
    validate_user_business_permission, validate_user_permission,
};
use crate::routes::department::schemas::DepartmentAccount;
use crate::routes::department::utils::{
    fetch_department_account_model_by_id, get_department_account,
    validate_department_account_active, validate_user_department_permission,
};
use crate::routes::user::schemas::{UserAccount, UserRoleType};
use crate::routes::user::utils::get_user;
use crate::schemas::{AllowedPermission, RequestMetaData, Status};
use crate::utils::{decode_token, get_header_value};
use actix_http::header::UPGRADE;
use actix_http::{Payload, h1};
use actix_web::body::{self, BoxBody, EitherBody, MessageBody};
use actix_web::dev::{Service, ServiceRequest, ServiceResponse, Transform, forward_ready};
use actix_web::error::PayloadError;
use actix_web::web::{Bytes, BytesMut};
use actix_web::{Error, HttpMessage, HttpResponseBuilder, ResponseError, http, web};
use core::str;
use futures::future::{self, LocalBoxFuture};
use futures::{Stream, StreamExt, stream};
use sqlx::PgPool;
use std::cell::RefCell;
use std::future::{Ready, ready};
use std::pin::Pin;
use std::rc::Rc;
use tracing::instrument;
use uuid::Uuid;

pub struct AuthMiddleware<S> {
    service: Rc<S>,
    allow_deleted_user: bool,
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
        let allow_deleted_user = self.allow_deleted_user;
        Box::pin(async move {
            let db_pool = &req.app_data::<web::Data<PgPool>>().unwrap();
            let user = get_user(vec![&decoded_user_id.to_string()], db_pool)
                .await
                .map_err(GenericError::UnexpectedError)?
                .ok_or(GenericError::DataNotFound("User not found".to_string()))?;
            if user.is_active == Status::Inactive {
                return Err(GenericError::ValidationError(
                    "User is Inactive. Please contact customer support".to_string(),
                ))?;
            } else if user.is_deleted & !allow_deleted_user {
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
pub struct RequireAuth {
    pub allow_deleted_user: bool,
}

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
            allow_deleted_user: self.allow_deleted_user,
        }))
    }
}

//Middleware to validate the business account
pub struct BusinessAccountMiddleware<S> {
    service: Rc<S>,
}
impl<S> Service<ServiceRequest> for BusinessAccountMiddleware<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<actix_web::body::BoxBody>, Error = Error>
        + 'static,
{
    type Response = ServiceResponse<actix_web::body::BoxBody>;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Error>>;

    forward_ready!(service);

    /// Handles incoming requests.
    #[instrument(skip(self), name = "business Account Middleware", fields(path = %req.path()))]
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

            if let Some(business_id) =
                get_header_value(&req, "x-business-id") // Convert HeaderValue to &str
                    .and_then(|value| Uuid::parse_str(value).ok())
            {
                let business_account = if user_account.user_role
                    != UserRoleType::Superadmin.to_string()
                {
                    get_business_account(db_pool, user_account.id, business_id)
                        .await
                        .map_err(|e| {
                            GenericError::DatabaseError(
                                "Something went wrong while fetching business account".to_string(),
                                e,
                            )
                        })?
                } else {
                    fetch_business_account_model_by_id(db_pool, Some(business_id))
                        .await
                        .map_err(|e| {
                            GenericError::DatabaseError(
                                "Something went wrong while fetching business account".to_string(),
                                e,
                            )
                        })?
                        .into_iter()
                        .next()
                        .map(|model| model.into_schema())
                };
                let extracted_business_account = business_account.ok_or_else(|| {
                    GenericError::ValidationError("business Account doesn't exist".to_string())
                })?;

                let error_message = validate_business_account_active(&extracted_business_account);
                if let Some(message) = error_message {
                    let (request, _pl) = req.into_parts();
                    let json_error = GenericError::ValidationError(message);
                    return Ok(ServiceResponse::from_err(json_error, request));
                }
                req.extensions_mut()
                    .insert::<BusinessAccount>(extracted_business_account);
            } else {
                let (request, _pl) = req.into_parts();
                return Ok(ServiceResponse::from_err(
                    GenericError::ValidationError("Please set x-business-id".to_string()),
                    request,
                ));
            }

            let res = srv.call(req).await?;
            Ok(res)
        })
    }
}

pub struct BusinessAccountValidation;

impl<S> Transform<S, ServiceRequest> for BusinessAccountValidation
where
    S: Service<ServiceRequest, Response = ServiceResponse<actix_web::body::BoxBody>, Error = Error>
        + 'static,
{
    type Response = ServiceResponse<actix_web::body::BoxBody>;
    type Error = Error;
    type Transform = BusinessAccountMiddleware<S>;
    type InitError = ();
    type Future = Ready<Result<Self::Transform, Self::InitError>>;
    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(BusinessAccountMiddleware {
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

pub struct BusinessPermissionMiddleware<S> {
    service: Rc<S>,
    pub permission_list: Vec<String>,
}
impl<S> Service<ServiceRequest> for BusinessPermissionMiddleware<S>
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
            let business_account = req
                .extensions()
                .get::<BusinessAccount>()
                .ok_or_else(|| {
                    GenericError::ValidationError(
                        "business Account Account doesn't exist".to_string(),
                    )
                })?
                .to_owned();
            // tracing::info!(
            //     "user_id: {:?}, business_id: {:?}, permission_list: {:?}",
            //     user_account.id,
            //     business_account.id,
            //     permission_list
            // );
            let permission_list = validate_user_business_permission(
                db_pool,
                user_account.id,
                business_account.id,
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

// Middleware factory for business account validation.
pub struct BusinessPermissionValidation {
    pub permission_list: Vec<String>,
}

impl<S> Transform<S, ServiceRequest> for BusinessPermissionValidation
where
    S: Service<ServiceRequest, Response = ServiceResponse<actix_web::body::BoxBody>, Error = Error>
        + 'static,
{
    type Response = ServiceResponse<actix_web::body::BoxBody>;
    type Error = Error;
    type Transform = BusinessPermissionMiddleware<S>;
    type InitError = ();
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    /// Creates and returns a new AuthMiddleware wrapped in a Result.
    fn new_transform(&self, service: S) -> Self::Future {
        // Wrap the AuthMiddleware instance in a Result and return it.
        ready(Ok(BusinessPermissionMiddleware {
            service: Rc::new(service),
            permission_list: self.permission_list.clone(),
        }))
    }
}

pub struct UserPermissionMiddleware<S> {
    service: Rc<S>,
    pub permission_list: Vec<String>,
}
impl<S> Service<ServiceRequest> for UserPermissionMiddleware<S>
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

            let permission_list =
                validate_user_permission(db_pool, user_account.id, &permission_list)
                    .await
                    .map_err(|e| {
                        GenericError::DatabaseError(
                            "Something went wrong while fetching user permission".to_owned(),
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

// Middleware factory for business account validation.
pub struct UserPermissionValidation {
    pub permission_list: Vec<String>,
}

impl<S> Transform<S, ServiceRequest> for UserPermissionValidation
where
    S: Service<ServiceRequest, Response = ServiceResponse<actix_web::body::BoxBody>, Error = Error>
        + 'static,
{
    type Response = ServiceResponse<actix_web::body::BoxBody>;
    type Error = Error;
    type Transform = UserPermissionMiddleware<S>;
    type InitError = ();
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    /// Creates and returns a new AuthMiddleware wrapped in a Result.
    fn new_transform(&self, service: S) -> Self::Future {
        // Wrap the AuthMiddleware instance in a Result and return it.
        ready(Ok(UserPermissionMiddleware {
            service: Rc::new(service),
            permission_list: self.permission_list.clone(),
        }))
    }
}

pub struct ReadReqResMiddleware2<S> {
    service: Rc<RefCell<S>>,
}

impl<S, B> Service<ServiceRequest> for ReadReqResMiddleware2<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: MessageBody + 'static,
    <B as MessageBody>::Error: ResponseError + 'static,
{
    type Response = ServiceResponse<EitherBody<B>>;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    forward_ready!(service);

    #[instrument(skip(self), name = "Request Response Payload", fields(path = %req.path()))]
    fn call(&self, mut req: ServiceRequest) -> Self::Future {
        let svc = self.service.clone();
        //
        let is_websocket = req.headers().contains_key(UPGRADE)
            && req.headers().get(UPGRADE).unwrap() == "websocket";
        let is_on_search = req.path().ends_with("on_search");
        if is_websocket || is_on_search {
            Box::pin(async move {
                let fut: ServiceResponse<B> = svc.call(req).await?;
                Ok(fut.map_into_left_body())
            })
        } else {
            Box::pin(async move {
                // let route = req.path().to_owned();

                let mut request_body = BytesMut::new();

                while let Some(chunk) = req.take_payload().next().await {
                    request_body.extend_from_slice(&chunk?);
                }
                let body = request_body.freeze();
                match str::from_utf8(&body) {
                    Ok(request_str) => {
                        if let Ok(request_json) =
                            // tracing::Span::current().record("Request body", &tracing::field::display("Apple"));
                            serde_json::from_str::<serde_json::Value>(request_str)
                        {
                            tracing::info!({%request_json}, "HTTP Response");
                        } else {
                            tracing::info!("Non-JSON request: {}", request_str);
                            request_str.to_string();
                        }
                    }

                    Err(_) => {
                        tracing::error!("Something went wrong in request body parsing middleware");
                    }
                }
                let single_part: Result<Bytes, PayloadError> = Ok(body);
                let in_memory_stream = stream::once(future::ready(single_part));
                let pinned_stream: Pin<Box<dyn Stream<Item = Result<Bytes, PayloadError>>>> =
                    Box::pin(in_memory_stream);
                let in_memory_payload: Payload = pinned_stream.into();
                req.set_payload(in_memory_payload);
                let fut = svc.call(req).await?;

                let res_status = fut.status();
                let res_headers = fut.headers().clone();
                let new_request = fut.request().clone();
                let mut new_response = HttpResponseBuilder::new(res_status);
                let body_bytes = body::to_bytes(fut.into_body()).await?;
                match str::from_utf8(&body_bytes) {
                    Ok(response_str) => {
                        if let Ok(response_json) =
                            serde_json::from_str::<serde_json::Value>(response_str)
                        {
                            tracing::info!({%response_json}, "HTTP Response");
                            tracing::Span::current()
                                .record("Response body", tracing::field::display(&response_str));

                            response_str.to_string()
                        } else {
                            // Not JSON, log as a string
                            tracing::info!("Non-JSON response: {}", response_str);
                            response_str.to_string()
                        }
                    }
                    Err(_) => {
                        tracing::error!("Something went wrong in response body parsing middleware");
                        "Something went wrong in response response body parsing middleware".into()
                    }
                };
                for (header_name, header_value) in res_headers {
                    new_response.insert_header((header_name.as_str(), header_value));
                }
                let new_response = new_response.body(body_bytes.to_vec());
                // Create the new ServiceResponse
                Ok(ServiceResponse::new(
                    new_request,
                    new_response.map_into_right_body(),
                ))

                // }
            })
        }
    }
}

pub struct SaveRequestResponse2;

impl<S, B> Transform<S, ServiceRequest> for SaveRequestResponse2
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: MessageBody + 'static,
    <B as MessageBody>::Error: ResponseError + 'static,
{
    type Response = ServiceResponse<EitherBody<B>>;
    type Error = Error;
    type InitError = ();
    type Transform = ReadReqResMiddleware2<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(ReadReqResMiddleware2 {
            service: Rc::new(RefCell::new(service)),
        }))
    }
}

pub struct DepartmentPermissionMiddleware<S> {
    service: Rc<S>,
    pub permission_list: Vec<String>,
}
impl<S> Service<ServiceRequest> for DepartmentPermissionMiddleware<S>
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
            let business_account = req
                .extensions()
                .get::<BusinessAccount>()
                .ok_or_else(|| {
                    GenericError::ValidationError(
                        "Business Account Account doesn't exist".to_string(),
                    )
                })?
                .to_owned();

            let department_account = req
                .extensions()
                .get::<DepartmentAccount>()
                .ok_or_else(|| {
                    GenericError::ValidationError(
                        "Department Account Account doesn't exist".to_string(),
                    )
                })?
                .to_owned();

            let permission_list = validate_user_department_permission(
                db_pool,
                user_account.id,
                business_account.id,
                department_account.id,
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

// Middleware factory for business account validation.
pub struct DepartmentPermissionValidation {
    pub permission_list: Vec<String>,
}

impl<S> Transform<S, ServiceRequest> for DepartmentPermissionValidation
where
    S: Service<ServiceRequest, Response = ServiceResponse<actix_web::body::BoxBody>, Error = Error>
        + 'static,
{
    type Response = ServiceResponse<actix_web::body::BoxBody>;
    type Error = Error;
    type Transform = DepartmentPermissionMiddleware<S>;
    type InitError = ();
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    /// Creates and returns a new AuthMiddleware wrapped in a Result.
    fn new_transform(&self, service: S) -> Self::Future {
        // Wrap the AuthMiddleware instance in a Result and return it.
        ready(Ok(DepartmentPermissionMiddleware {
            service: Rc::new(service),
            permission_list: self.permission_list.clone(),
        }))
    }
}

//Middleware to validate the business account
pub struct DepartmentAccountMiddleware<S> {
    service: Rc<S>,
}
impl<S> Service<ServiceRequest> for DepartmentAccountMiddleware<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<actix_web::body::BoxBody>, Error = Error>
        + 'static,
{
    type Response = ServiceResponse<actix_web::body::BoxBody>;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Error>>;

    forward_ready!(service);

    /// Handles incoming requests.
    #[instrument(skip(self), name = "business Account Middleware", fields(path = %req.path()))]
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

            let business_account = req
                .extensions()
                .get::<BusinessAccount>()
                .ok_or_else(|| {
                    GenericError::ValidationError("Business Account doesn't exist".to_string())
                })?
                .to_owned();

            if let Some(department_id) =
                get_header_value(&req, "x-department-id") // Convert HeaderValue to &str
                    .and_then(|value| Uuid::parse_str(value).ok())
            {
                let department_account = if user_account.user_role
                    != UserRoleType::Superadmin.to_string()
                {
                    get_department_account(
                        db_pool,
                        user_account.id,
                        business_account.id,
                        department_id,
                    )
                    .await
                    .map_err(|e| {
                        GenericError::DatabaseError(
                            "Something went wrong while fetching department account".to_string(),
                            e,
                        )
                    })?
                } else {
                    fetch_department_account_model_by_id(db_pool, Some(department_id))
                        .await
                        .map_err(|e| {
                            GenericError::DatabaseError(
                                "Something went wrong while fetching  account".to_string(),
                                e,
                            )
                        })?
                        .into_iter()
                        .next()
                        .map(|model| model.into_schema())
                };
                let extracted_department_account = department_account.ok_or_else(|| {
                    GenericError::ValidationError("Department Account doesn't exist".to_string())
                })?;

                let error_message =
                    validate_department_account_active(&extracted_department_account);
                if let Some(message) = error_message {
                    let (request, _pl) = req.into_parts();
                    let json_error = GenericError::ValidationError(message);
                    return Ok(ServiceResponse::from_err(json_error, request));
                }
                req.extensions_mut()
                    .insert::<DepartmentAccount>(extracted_department_account);
            } else {
                let (request, _pl) = req.into_parts();
                return Ok(ServiceResponse::from_err(
                    GenericError::ValidationError("Please set x-department-id".to_string()),
                    request,
                ));
            }

            let res = srv.call(req).await?;
            Ok(res)
        })
    }
}

pub struct DepartmentAccountValidation;

impl<S> Transform<S, ServiceRequest> for DepartmentAccountValidation
where
    S: Service<ServiceRequest, Response = ServiceResponse<actix_web::body::BoxBody>, Error = Error>
        + 'static,
{
    type Response = ServiceResponse<actix_web::body::BoxBody>;
    type Error = Error;
    type Transform = DepartmentAccountMiddleware<S>;
    type InitError = ();
    type Future = Ready<Result<Self::Transform, Self::InitError>>;
    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(DepartmentAccountMiddleware {
            service: Rc::new(service),
        }))
    }
}
