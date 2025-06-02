use std::fmt;
use std::future::{Ready, ready};

use crate::email::{EmailObject, deserialize_subscriber_email};
// use crate::routes::business::schemas::BasicBusinessAccount;
use crate::errors::GenericError;
use crate::schemas::{MaskingType, Status};
use crate::utils::pascal_to_snake_case;
use actix_http::Payload;
use actix_web::{FromRequest, HttpMessage, HttpRequest, web};
use chrono::{DateTime, Utc};
use futures::future::LocalBoxFuture;
use secrecy::{ExposeSecret, SecretString};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(Serialize, Deserialize, Debug, sqlx::Type, PartialEq, ToSchema)]
#[sqlx(type_name = "user_type", rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum RoleType {
    Guest,
    Developer,
    Maintainer,
    Agent,
    Superadmin,
    Admin,
}

impl fmt::Display for RoleType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl RoleType {
    pub fn to_lowercase_string(&self) -> String {
        format!("{:?}", self).to_lowercase()
    }
}

#[derive(Deserialize, Debug, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct CreateUserAccount {
    pub username: String,
    pub mobile_no: String,
    pub international_dialing_code: String,
    #[schema(value_type = String)]
    pub password: SecretString,
    #[serde(deserialize_with = "deserialize_subscriber_email")]
    pub email: EmailObject,
    pub display_name: String,
    pub is_test_user: bool,
    pub user_type: RoleType,
}

impl CreateUserAccount {
    pub fn get_full_mobile_no(&self) -> String {
        format!("{}{}", self.international_dialing_code, self.mobile_no)
    }
}

impl FromRequest for CreateUserAccount {
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
pub struct AuthenticateRequest {
    pub scope: AuthenticationScope,
    pub identifier: String,
    #[schema(value_type = String)]
    pub secret: SecretString,
}

impl FromRequest for AuthenticateRequest {
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

// #[derive(Serialize, Deserialize, Debug)]
// #[serde(rename_all = "snake_case")]
// pub enum CreateUserType {
//     Guest,
//     Member,
//     Developer,
//     Maintainer,
// }
// impl From<CreateUserType> for UserType {
//     fn from(create_user_type: CreateUserType) -> Self {
//         match create_user_type {
//             CreateUserType::Guest => UserType::Guest,
//             CreateUserType::Maintainer => UserType::Maintainer,
//             CreateUserType::Developer => UserType::Developer,
//             CreateUserType::Agent => UserType::Agent,
//         }
//     }
// }

#[derive(Serialize, Deserialize, Debug, sqlx::Type, ToSchema, PartialEq)]
#[serde(rename_all = "snake_case")]
#[sqlx(type_name = "user_auth_identifier_scope", rename_all = "snake_case")]
pub enum AuthenticationScope {
    Otp,
    Password,
    Google,
    Facebook,
    Microsoft,
    Apple,
    Token,
    AuthApp,
    Qr,
    Email,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct AuthData {
    pub user: UserAccount,
    #[serde(serialize_with = "round_serialize")]
    #[schema(value_type = String)]
    pub token: SecretString,
    // pub business_account_list: Vec<BasicBusinessAccount>,
}

fn round_serialize<S>(x: &SecretString, s: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    s.serialize_str(x.expose_secret())
}

#[derive(Serialize, Deserialize, Debug, sqlx::Type, ToSchema)]
#[sqlx(type_name = "auth_context_type", rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum AuthContextType {
    UserAccount,
    BusinessAccount,
}

#[allow(dead_code)]
#[derive(Debug)]
pub struct AuthMechanism {
    pub id: Uuid,
    pub user_id: Uuid,
    pub auth_scope: AuthenticationScope,
    pub auth_identifier: String,
    pub secret: Option<SecretString>,
    pub is_active: Status,
    pub valid_upto: Option<DateTime<Utc>>,
    // pub auth_context: AuthContextType,
}

#[allow(dead_code)]
pub struct AccountRole {
    pub id: Uuid,
    pub role_name: String,
    pub role_status: Status,
    pub is_deleted: bool,
}

#[derive(Debug, Serialize)]
pub struct BulkAuthMechanismInsert {
    pub id: Vec<Uuid>,
    pub user_id_list: Vec<Uuid>,
    pub auth_scope: Vec<AuthenticationScope>,
    // #[serde(borrow)]
    pub auth_identifier: Vec<String>,
    pub secret: Vec<String>,
    pub is_active: Vec<Status>,
    pub created_on: Vec<DateTime<Utc>>,
    pub created_by: Vec<Uuid>,
    // pub auth_context: Vec<AuthContextType>,
}

#[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct UserAccount {
    #[schema(value_type = String)]
    pub id: Uuid,
    pub username: String,
    pub mobile_no: String,
    pub email: EmailObject,
    pub is_active: Status,
    pub display_name: String,
    pub vectors: Vec<UserVector>,
    pub international_dialing_code: String,
    pub is_test_user: bool,
    pub is_deleted: bool,
    pub user_role: String,
}

impl UserAccount {
    // pub fn get_ws_parms(&self) -> WebSocketParam {
    //     WebSocketParam {
    //         user_id: Some(self.id),
    //         business_id: None,
    //         device_id: None,
    //     }
    // }

    fn get_vector(&self, vector_type: VectorType) -> Option<&UserVector> {
        self.vectors.iter().find(|a| a.key == vector_type)
    }

    pub fn is_vector_verified(&self, vector_type: VectorType) -> bool {
        self.get_vector(vector_type).is_some_and(|f| f.verified)
    }
}

impl FromRequest for UserAccount {
    type Error = GenericError;
    type Future = Ready<Result<Self, Self::Error>>;
    fn from_request(
        req: &actix_web::HttpRequest,
        _payload: &mut actix_web::dev::Payload,
    ) -> Self::Future {
        let value = req.extensions().get::<UserAccount>().cloned();

        let result = match value {
            Some(user) => Ok(user),
            None => Err(GenericError::UnexpectedCustomError(
                "Something went wrong while parsing user account detail".to_string(),
            )),
        };

        ready(result)
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct JWTClaims {
    pub sub: Uuid,
    pub exp: usize,
}

#[derive(Serialize, Deserialize, Debug, sqlx::Type, PartialEq, Clone, ToSchema)]
#[sqlx(type_name = "vector_type", rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum VectorType {
    PanCardNo,
    Gstin,
    AadhaarCardNo,
    MobileNo,
    Email,
    InternationalDialingCode,
    UpiId,
    BankAccountNumber,
    IfscCode,
    LicenseNumber,
    PassportNo,
    VoterIdNo,
    Ssn,
    Tin,
    ExportLicenseNo,
    FssaiLicenseNumber,
    ImportLicenseNo,
}

impl std::fmt::Display for VectorType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", pascal_to_snake_case(&format!("{:?}", self)))
    }
}

#[derive(Serialize, Deserialize, Debug, sqlx::Type, Clone, ToSchema)]
#[sqlx(type_name = "vectors")]
pub struct UserVector {
    pub key: VectorType,
    pub value: String,
    pub masking: MaskingType,
    pub verified: bool,
}

#[derive(Deserialize, Debug, ToSchema)]
#[serde(rename_all = "camelCase")]
#[allow(dead_code)]
pub struct SendOTPRequest {
    pub mobile_no: String,
}

impl FromRequest for SendOTPRequest {
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
