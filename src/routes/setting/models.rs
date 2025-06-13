use chrono::{DateTime, Utc};
use sqlx::{FromRow, types::Json};
use uuid::Uuid;

use super::schemas::SettingEnumData;

#[derive(Debug)]
pub struct BulkSettingCreateModel {
    pub id_list: Vec<Uuid>,
    pub user_id_list: Vec<Option<Uuid>>,
    pub project_id_list: Vec<Option<Uuid>>,
    pub setting_id_list: Vec<Uuid>,
    pub value_list: Vec<String>,
    pub created_on_list: Vec<DateTime<Utc>>,
    pub created_by_list: Vec<Uuid>,
}

#[derive(Debug, FromRow)]
#[allow(dead_code)]
pub struct SettingModel {
    pub id: Uuid,
    pub key: String,
    pub is_editable: bool,
    pub enum_id: Option<Uuid>,
}

#[derive(Debug, FromRow)]
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

#[derive(FromRow, Debug)]
pub struct SettingEnumModel {
    pub id: Uuid,
    pub values: Json<Vec<String>>,
}

impl SettingEnumModel {
    pub fn into_schema(self) -> SettingEnumData {
        SettingEnumData {
            id: self.id,
            value_list: self.values.to_vec(),
        }
    }
}
