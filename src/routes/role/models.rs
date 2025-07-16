use chrono::{DateTime, Utc};
use sqlx::FromRow;
use uuid::Uuid;

use crate::schemas::Status;

use super::schemas::AccountRole;

#[allow(dead_code)]
#[derive(Debug, FromRow)]
pub struct RoleModel {
    pub id: Uuid,
    pub name: String,
    pub status: Status,
    pub created_on: DateTime<Utc>,
    pub created_by: Uuid,
    pub is_deleted: bool,
    pub is_editable: bool,
}

impl RoleModel {
    pub fn into_schema(self) -> AccountRole {
        AccountRole {
            id: self.id,
            name: self.name,
            status: self.status,
            is_deleted: self.is_deleted,
            is_editable: self.is_editable,
        }
    }
}
