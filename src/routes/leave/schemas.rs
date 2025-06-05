use std::fmt;

use crate::{email::EmailObject, errors::GenericError, routes::setting::schemas::SettingKey};
use actix_http::Payload;
use actix_web::{FromRequest, HttpRequest, web};
use chrono::{DateTime, Utc};
use futures::future::LocalBoxFuture;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(Serialize, Deserialize, Debug, ToSchema, sqlx::Type)]
#[sqlx(type_name = "leave_type", rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum LeaveType {
    Medical,
    Casual,
    Restricted,
    Common, // Global
}

impl fmt::Display for LeaveType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let display_str = match self {
            LeaveType::Medical => "medical",
            LeaveType::Casual => "casual",
            LeaveType::Restricted => "restricted",
            LeaveType::Common => "common",
        };
        write!(f, "{}", display_str)
    }
}

impl LeaveType {
    pub fn get_setting_key(&self) -> SettingKey {
        match self {
            LeaveType::Medical => SettingKey::TotalMedicalLeaveCount,
            LeaveType::Casual => SettingKey::TotalCasualLeaveCount,
            LeaveType::Restricted => SettingKey::TotalRestrictedLeaveCount,
            LeaveType::Common => SettingKey::TotalCommonLeaveCount,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, ToSchema, sqlx::Type, PartialEq, Eq, Hash)]
#[sqlx(type_name = "leave_period", rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum LeavePeriod {
    HalfDay,
    FullDay,
}

#[derive(Deserialize, Debug, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct CreateLeaveData {
    pub period: LeavePeriod,
    pub date: chrono::NaiveDate,
}

#[derive(Serialize, Deserialize, Debug, sqlx::Type)]
#[sqlx(type_name = "leave_status", rename_all = "snake_case")]
pub enum LeaveStatus {
    Approved,
    Rejected,
    Cancelled,
    Requested,
}

#[derive(Deserialize, Debug, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct CreateLeaveRequest {
    pub to: EmailObject,
    pub cc: Option<Vec<EmailObject>>,
    pub reason: Option<String>,
    pub r#type: LeaveType,
    #[schema(value_type = String)]
    pub user_id: Option<Uuid>,
    pub leave_data: Vec<CreateLeaveData>,
}

impl FromRequest for CreateLeaveRequest {
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
pub struct BulkLeaveRequestInsert<'a> {
    pub id: Vec<Uuid>,
    pub user_id_list: Vec<Uuid>,
    pub created_on: Vec<DateTime<Utc>>,
    pub created_by: Vec<Uuid>,
    pub leave_type: Vec<&'a LeaveType>,
    pub leave_period: Vec<&'a LeavePeriod>,
    pub date: Vec<DateTime<Utc>>,
    pub status: Vec<LeaveStatus>,
    pub reason: Vec<Option<&'a str>>,
    pub email_message_id: Vec<Option<&'a str>>,
}

#[derive(Serialize)]
pub struct LeaveRequestEmailContext<'a> {
    name: &'a str,
    dates: Vec<String>,
    reason: &'a str,
    receiver: &'a str,
}

impl<'a> LeaveRequestEmailContext<'a> {
    pub fn new(name: &'a str, dates: Vec<String>, reason: &'a str, receiver: &'a str) -> Self {
        Self {
            name,
            dates,
            reason,
            receiver,
        }
    }
}
