use chrono::{DateTime, Utc};
use serde::Deserialize;
use sqlx::FromRow;

use super::schemas::{LeavePeriod, LeaveType};

#[derive(Deserialize, Debug, FromRow)]
#[serde(rename_all = "camelCase")]
#[allow(dead_code)]
pub struct LeaveDataModel {
    pub r#type: LeaveType,
    pub period: LeavePeriod,
    pub date: Vec<DateTime<Utc>>,
    pub reason: Option<String>,
}
