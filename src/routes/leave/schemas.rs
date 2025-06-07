use std::fmt;

use crate::{email::EmailObject, errors::GenericError, routes::setting::schemas::SettingKey};
use actix_http::Payload;
use actix_web::{FromRequest, HttpRequest, web};
use chrono::{DateTime, FixedOffset, NaiveDateTime, Utc};
use chrono_tz::Tz;
use futures::future::LocalBoxFuture;
use serde::{Deserialize, Serialize};
use serde_json::Value;
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

#[derive(Serialize, Deserialize, Debug, sqlx::Type, ToSchema, PartialEq)]
#[sqlx(type_name = "leave_status", rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
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
    pub sender_id: Vec<Uuid>,
    pub receiver_id: Vec<Uuid>,
    pub created_on: Vec<DateTime<Utc>>,
    pub created_by: Vec<Uuid>,
    pub leave_type: Vec<&'a LeaveType>,
    pub leave_period: Vec<&'a LeavePeriod>,
    pub date: Vec<DateTime<Utc>>,
    pub status: Vec<LeaveStatus>,
    pub reason: Vec<Option<&'a str>>,
    pub email_message_id: Vec<Option<&'a str>>,
    pub cc: Vec<Option<Value>>,
}

#[derive(Serialize)]
pub struct LeaveRequestEmailContext<'a> {
    name: &'a str,
    dates: Vec<String>,
    reason: &'a str,
    receiver: &'a str,
    r#type: &'a LeaveType,
}

impl<'a> LeaveRequestEmailContext<'a> {
    pub fn new(
        name: &'a str,
        dates: Vec<String>,
        reason: &'a str,
        receiver: &'a str,
        r#type: &'a LeaveType,
    ) -> Self {
        Self {
            name,
            dates,
            reason,
            receiver,
            r#type,
        }
    }
}

#[derive(Deserialize, Debug, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct UpdateLeaveStatusRequest {
    pub id: Uuid,
    pub status: LeaveStatus,
}

impl FromRequest for UpdateLeaveStatusRequest {
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

#[derive(Deserialize, Debug, ToSchema, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LeaveData {
    pub id: Uuid,
    pub r#type: LeaveType,
    pub period: LeavePeriod,
    pub date: DateTime<Utc>,
    pub reason: Option<String>,
    pub status: LeaveStatus,
    pub sender_id: Uuid,
    pub email_message_id: Option<String>,
    pub cc: Option<Vec<EmailObject>>,
    pub created_on: Option<DateTime<FixedOffset>>,
}

#[derive(Serialize)]
pub struct LeaveRequestStatusEmailContext<'a> {
    name: &'a str,
    receiver: &'a str,
    status: &'a LeaveStatus,
    pub date: &'a DateTime<Utc>,
}

impl<'a> LeaveRequestStatusEmailContext<'a> {
    pub fn new(
        name: &'a str,
        receiver: &'a str,
        status: &'a LeaveStatus,
        date: &'a DateTime<Utc>,
    ) -> Self {
        Self {
            name,
            receiver,
            status,
            date,
        }
    }
}
#[derive(Deserialize, Debug, ToSchema, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum FetchLeaveType {
    Sender,
    Receiver,
}

#[derive(Deserialize, Debug, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct FetchLeaveRequest {
    pub action_type: FetchLeaveType,
    pub id: Option<Uuid>,
    pub limit: i32,
    pub offset: i32,
    pub start_date: Option<NaiveDateTime>,
    pub end_date: Option<NaiveDateTime>,
}

impl FromRequest for FetchLeaveRequest {
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
pub struct FetchLeaveQuery<'a> {
    pub date: Option<&'a DateTime<Utc>>,
    pub period: Option<&'a LeavePeriod>,
    pub sender_id: Option<Uuid>,
    pub leave_id: Option<Uuid>,
    pub offset: Option<i32>,
    pub limit: Option<i32>,
    pub start_date: Option<&'a NaiveDateTime>,
    pub end_date: Option<&'a NaiveDateTime>,
    pub tz: Option<&'a Tz>,
    pub receiver_id: Option<Uuid>,
}

impl<'a> FetchLeaveQuery<'a> {
    pub fn builder() -> Self {
        Self {
            date: None,
            period: None,
            sender_id: None,
            leave_id: None,
            offset: None,
            limit: None,
            start_date: None,
            end_date: None,
            tz: None,
            receiver_id: None,
        }
    }

    pub fn with_sender_id(mut self, sender_id: Option<Uuid>) -> Self {
        self.sender_id = sender_id;
        self
    }
    pub fn with_leave_id(mut self, leave_id: Option<Uuid>) -> Self {
        self.leave_id = leave_id;
        self
    }
    pub fn with_recevier_id(mut self, receiver_id: Option<Uuid>) -> Self {
        self.receiver_id = receiver_id;
        self
    }
    pub fn with_offset(mut self, offset: Option<i32>) -> Self {
        self.offset = offset;
        self
    }

    pub fn with_limit(mut self, limit: Option<i32>) -> Self {
        self.limit = limit;
        self
    }

    #[allow(dead_code)]
    pub fn with_date(mut self, date: Option<&'a DateTime<Utc>>) -> Self {
        self.date = date;
        self
    }

    pub fn with_start_date(mut self, date: Option<&'a NaiveDateTime>) -> Self {
        self.start_date = date;
        self
    }
    pub fn with_end_date(mut self, date: Option<&'a NaiveDateTime>) -> Self {
        self.end_date = date;
        self
    }

    #[allow(dead_code)]
    pub fn with_period(mut self, period: Option<&'a LeavePeriod>) -> Self {
        self.period = period;
        self
    }
    pub fn with_tz(mut self, tz: Option<&'a Tz>) -> Self {
        self.tz = tz;
        self
    }
}
