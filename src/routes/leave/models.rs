use crate::email::EmailObject;

use super::schemas::{LeaveData, LeavePeriod, LeaveStatus, LeaveType};
use chrono::{DateTime, TimeZone, Utc};
use chrono_tz::Tz;

use sqlx::{FromRow, types::Json};
use uuid::Uuid;

#[derive(Debug, FromRow)]
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
    pub created_on: DateTime<Utc>,
}

impl LeaveDataModel {
    pub fn into_schema(self, time_zone: Option<&Tz>) -> LeaveData {
        let local_dt = time_zone.map(|tz| {
            tz.from_utc_datetime(&self.created_on.naive_utc())
                .fixed_offset()
        });

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
            created_on: local_dt,
        }
    }
}
