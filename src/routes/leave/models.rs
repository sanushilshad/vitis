use crate::email::EmailObject;

use super::schemas::{
    LeaveData, LeaveGroup, LeavePeriod, LeaveStatus, LeaveTypeData, UserLeave, UserLeaveGroup,
    UserLeaveType,
};
use bigdecimal::BigDecimal;
use chrono::{DateTime, TimeZone, Utc};
use chrono_tz::Tz;

use sqlx::{FromRow, types::Json};
use uuid::Uuid;

#[derive(Debug, FromRow)]
pub struct LeaveDataModel {
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

#[derive(Debug, FromRow)]
pub struct MinimalLeaveModel {
    pub id: Uuid,
    pub period: LeavePeriod,
    pub sender_id: Uuid,
}

#[derive(Debug, FromRow)]
pub struct LeaveTypeModel {
    pub id: Uuid,
    pub label: String,
}

impl LeaveTypeModel {
    pub fn into_schema(self) -> LeaveTypeData {
        LeaveTypeData {
            id: self.id,
            label: self.label,
        }
    }
}

#[derive(Debug, FromRow)]
pub struct LeaveGroupModel {
    pub id: Uuid,
    pub label: String,
    pub start_date: DateTime<Utc>,
    pub end_date: DateTime<Utc>,
}

impl LeaveGroupModel {
    pub fn into_schema(self) -> LeaveGroup {
        LeaveGroup {
            id: self.id,
            label: self.label,
            start_date: self.start_date,
            end_date: self.end_date,
        }
    }
}

#[derive(Debug, FromRow)]
pub struct UserLeaveModel {
    pub id: Uuid,
    pub allocated_count: BigDecimal,
    pub used_count: BigDecimal,
    pub business_id: Uuid,
    pub user_id: Uuid,
    pub leave_type_id: Uuid,
    pub leave_group_id: Uuid,
    pub leave_group_label: String,
    pub leave_type_label: String,
}

impl UserLeaveModel {
    pub fn into_schema(self) -> UserLeave {
        UserLeave {
            id: self.id,
            allocated_count: self.allocated_count,
            used_count: self.used_count,
            business_id: self.business_id,
            user_id: self.user_id,
            leave_type: UserLeaveType {
                id: self.leave_type_id,
                label: self.leave_type_label,
            },
            leave_group: UserLeaveGroup {
                id: self.leave_group_id,
                label: self.leave_group_label,
            },
        }
    }
}
