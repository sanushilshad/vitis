use std::fmt;

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
pub struct CreateBusinessSettingRequest {
    pub user_id: Option<Uuid>,
    pub settings: Vec<CreateSettingData>,
}

impl FromRequest for CreateBusinessSettingRequest {
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
    pub user_id: Option<Uuid>,
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
    pub value: Option<String>,
    pub id: Option<Uuid>,
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
    pub business_level: Vec<Setting>,
}

impl Settings {
    pub fn compute_setting(&self) -> Option<String> {
        if !self.user_level.is_empty() {
            return self.user_level.first().and_then(|obj| obj.value.clone());
        }
        if !self.business_level.is_empty() {
            return self
                .business_level
                .first()
                .and_then(|obj| obj.value.clone());
        }
        if !self.global_level.is_empty() {
            return self.global_level.first().and_then(|obj| obj.value.clone());
        }
        None
    }
}

pub trait SettingsExt {
    fn get_setting(&self, key: &str) -> Option<String>;
}

impl SettingsExt for Vec<Settings> {
    fn get_setting(&self, key: &str) -> Option<String> {
        self.iter()
            .find(|setting| setting.key == key)
            .and_then(|setting| setting.compute_setting())
    }
}
#[derive(Serialize, Debug, ToSchema)]
pub struct SettingData {
    pub settings: Vec<Settings>,
}

#[derive(Deserialize, Debug, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct CreateUserSettingRequest {
    pub user_id: Option<Uuid>,
    pub settings: Vec<CreateSettingData>,
}

impl FromRequest for CreateUserSettingRequest {
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

#[derive(Deserialize, Debug, ToSchema, PartialEq)]
pub enum SettingType {
    Global,
    User,
    Business,
}

impl SettingType {
    // pub fn as_str(&self) -> &str {
    //     match self {
    //         // SettingType::Global => "global",
    //         SettingType::User => "user",
    //         SettingType::Business => "business",
    //     }
    // }
}

#[derive(Serialize, Debug)]
pub enum SettingKey {
    EmailAppPassword,
    TotalCommonLeaveCount,
    TotalRestrictedLeaveCount,
    TotalMedicalLeaveCount,
    TotalCasualLeaveCount,
    FinancialYearStart,
    LeaveRequestTemplate,
    LeaveStatusUpdateTemplate,
    TimeZone,
    UnpaidLeaveCount,
    EmailOTPTemplate,
}

impl fmt::Display for SettingKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let display_str = match self {
            SettingKey::EmailAppPassword => "email_app_password",
            SettingKey::TotalCommonLeaveCount => "total_common_leave_count",
            SettingKey::TotalRestrictedLeaveCount => "total_restricted_leave_count",
            SettingKey::TotalMedicalLeaveCount => "total_medical_leave_count",
            SettingKey::TotalCasualLeaveCount => "total_casual_leave_count",
            SettingKey::FinancialYearStart => "financial_year_start",
            SettingKey::LeaveRequestTemplate => "leave_request_template",
            SettingKey::LeaveStatusUpdateTemplate => "leave_status_update_template",
            SettingKey::TimeZone => "time_zone",
            SettingKey::UnpaidLeaveCount => "unpaid_leave_count",
            SettingKey::EmailOTPTemplate => "email_otp_template",
        };
        write!(f, "{}", display_str)
    }
}

#[derive(Serialize, Debug, ToSchema)]
pub struct SettingEnumData {
    pub id: Uuid,
    pub value_list: Vec<String>,
}

#[derive(Deserialize, Debug, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct FetchSettingEnumRequest {
    pub id_list: Vec<Uuid>,
}

impl FromRequest for FetchSettingEnumRequest {
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
