use sqlx::FromRow;
use uuid::Uuid;

use super::schemas::Permission;

#[derive(Debug, FromRow)]
pub struct PermissionModel {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
}
impl PermissionModel {
    pub fn into_schema(self) -> Permission {
        Permission {
            id: self.id,
            name: self.name,
            description: self.description,
        }
    }
}
