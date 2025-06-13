use chrono::{DateTime, Utc};
use serde::Deserialize;
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Deserialize, Debug)]
pub struct BulkSettingCreateModel {
    pub id_list: Vec<Uuid>,
    pub user_id_list: Vec<Option<Uuid>>,
    pub project_id_list: Vec<Option<Uuid>>,
    pub setting_id_list: Vec<Uuid>,
    pub value_list: Vec<String>,
    pub created_on_list: Vec<DateTime<Utc>>,
    pub created_by_list: Vec<Uuid>,
}

#[derive(Deserialize, Debug, FromRow)]
pub struct SettingModel {
    pub id: Uuid,
    pub key: String,
    pub is_editable: bool,
}

#[derive(Deserialize, Debug, FromRow)]
pub struct SettingValueModel {
    pub id: Option<Uuid>,
    pub key: String,
    pub value: Option<String>,
    pub label: String,
    pub enum_id: Option<Uuid>,
    pub user_id: Option<Uuid>,
    pub project_id: Option<Uuid>,
    pub is_editable: bool,
}
