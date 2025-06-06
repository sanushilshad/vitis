use crate::email::EmailObject;

use super::schemas::{LeaveData, LeavePeriod, LeaveStatus, LeaveType};
use chrono::{DateTime, Utc};
use serde::Deserialize;
use serde_json::Value;
use sqlx::{FromRow, types::Json};
use uuid::Uuid;

#[derive(Deserialize, Debug, FromRow)]
#[serde(rename_all = "camelCase")]
#[allow(dead_code)]
pub struct LeaveDataModel {
    pub r#type: LeaveType,
    pub period: LeavePeriod,
    pub date: DateTime<Utc>,
    pub reason: Option<String>,
    pub status: LeaveStatus,
    pub sender_id: Uuid,
    pub email_message_id: Option<String>,
    pub cc: Option<Json<Vec<EmailObject>>>,
    pub id: Uuid,
    pub is_deleted: bool,
}

impl LeaveDataModel {
    pub fn into_schema(self) -> LeaveData {
        LeaveData {
            r#type: self.r#type,
            period: self.period,
            date: self.date,
            reason: self.reason,
            status: self.status,
            sender_id: self.sender_id,
            email_message_id: self.email_message_id,
            cc: self.cc.map(|a| a.to_vec()),
            id: self.id,
            is_deleted: self.is_deleted,
        }
    }
}
