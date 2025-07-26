use crate::email::EmailObject;

use super::schemas::{
    LeaveAllowedDate, LeaveGroup, LeavePeriodData, LeaveRequestData, LeaveStatus, LeaveTypeData,
    UserLeave, UserLeaveGroup, UserLeaveType,
};
use bigdecimal::BigDecimal;
use chrono::{DateTime, NaiveDate, TimeZone, Utc};
use chrono_tz::Tz;

use serde::{Deserialize, Serialize};
use sqlx::{FromRow, types::Json};
use uuid::Uuid;

#[derive(Debug, FromRow)]
pub struct LeaveDataModel {
    pub date: DateTime<Utc>,
    pub reason: Option<String>,
    pub status: LeaveStatus,
    // pub sender_id: Uuid,
    pub email_message_id: Option<String>,
    pub cc: Option<Json<Vec<EmailObject>>>,
    pub id: Uuid,
    pub created_on: DateTime<Utc>,
    pub leave_type: String,
    pub user_id: Uuid,
    pub user_leave_id: Uuid,
    pub leave_period_id: Uuid,
    pub period_label: String,
    pub period_value: BigDecimal,
}

impl LeaveDataModel {
    pub fn into_schema(self, time_zone: Option<&Tz>) -> LeaveRequestData {
        let local_dt = time_zone.map(|tz| {
            tz.from_utc_datetime(&self.created_on.naive_utc())
                .fixed_offset()
        });

        LeaveRequestData {
            // period: self.period,
            date: self.date,
            reason: self.reason,
            status: self.status,
            // sender_id: self.sender_id,
            email_message_id: self.email_message_id,
            cc: self.cc.map(|a| a.to_vec()),
            id: self.id,
            created_on: local_dt,
            leave_type: self.leave_type,
            user_id: self.user_id,
            user_leave_id: self.user_leave_id,
            // leave_period_id: self.leave_period_id,
            period: LeavePeriodData {
                id: self.leave_period_id,
                label: self.period_label,
                value: self.period_value,
            },
        }
    }
}

#[derive(Debug, FromRow)]
pub struct MinimalLeaveModel {
    pub id: Uuid,

    pub period: String,
    pub user_id: Uuid,
    pub r#type: String,
}

// pub period_label: String,
// pub period_id: Uuid,

#[derive(Debug, FromRow, Deserialize, Serialize)]
pub struct LeaveAllowedDateModel {
    pub date: NaiveDate,
    pub label: String,
}

impl LeaveAllowedDateModel {
    pub fn into_schema(self) -> LeaveAllowedDate {
        LeaveAllowedDate {
            date: self.date,
            label: self.label,
        }
    }
}

#[derive(Debug, FromRow)]
pub struct LeaveTypeModel {
    pub id: Uuid,
    pub label: String,
    pub allowed_dates: Option<Json<Vec<LeaveAllowedDateModel>>>,
}

impl LeaveTypeModel {
    pub fn into_schema(self, periods: Vec<LeavePeriodData>) -> LeaveTypeData {
        LeaveTypeData {
            id: self.id,
            label: self.label,
            period_list: periods,
            allowed_dates: self
                .allowed_dates
                .map(|dates| dates.0.into_iter().map(|d| d.into_schema()).collect()),
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

#[derive(Debug, FromRow, Deserialize)]
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
    pub allowed_dates: Option<Json<Vec<LeaveAllowedDateModel>>>,
    // pub period_label: String,
    // pub period_id: Uuid,
}

impl UserLeaveModel {
    pub fn into_schema(self, periods: Vec<LeavePeriodData>) -> UserLeave {
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
            periods,
            allowed_dates: self
                .allowed_dates
                .map(|p| p.0.into_iter().map(|p| p.into_schema()).collect()),
        }
    }
}

#[derive(Debug, FromRow)]
pub struct LeavePeriodModel {
    pub id: Uuid,
    pub label: String,
    pub value: BigDecimal,
    // pub type_id: Uuid,
}

impl LeavePeriodModel {
    pub fn into_schema(self) -> LeavePeriodData {
        LeavePeriodData {
            id: self.id,
            label: self.label,
            value: self.value,
        }
    }
}

#[derive(Debug, FromRow, Clone, Deserialize)]
pub struct LeavePeriodWithTypeModel {
    pub id: Uuid,
    pub label: String,
    pub value: BigDecimal,
    pub type_id: Uuid,
}

impl LeavePeriodWithTypeModel {
    pub fn into_schema(self) -> LeavePeriodData {
        LeavePeriodData {
            id: self.id,
            label: self.label,
            value: self.value,
        }
    }
}
