use sqlx::FromRow;
use uuid::Uuid;

use crate::schemas::Status;

use super::schemas::{BasicDepartmentAccount, DepartmentAccount};
#[allow(dead_code)]
#[derive(Debug, FromRow)]
pub struct DepartmentAccountModel {
    pub id: Uuid,
    pub display_name: String,
    pub is_active: Status,
    pub is_deleted: bool,
    pub verified: bool,
}

impl DepartmentAccountModel {
    #[allow(dead_code)]
    pub fn into_basic_schema(self) -> BasicDepartmentAccount {
        BasicDepartmentAccount {
            id: self.id,
            name: self.display_name,
        }
    }
    #[allow(dead_code)]
    pub fn into_schema(self) -> DepartmentAccount {
        DepartmentAccount {
            id: self.id,
            display_name: self.display_name.to_string(),
            is_active: self.is_active.to_owned(),
            is_deleted: self.is_deleted,
            verified: self.verified,
        }
    }
}

#[derive(Debug, FromRow)]
#[allow(dead_code)]
pub struct UserDepartmentRelationAccountModel {
    pub id: Uuid,
    pub display_name: String,
    pub is_active: Status,
    pub verified: bool,
    pub is_deleted: bool,
}

impl UserDepartmentRelationAccountModel {
    #[allow(dead_code)]
    pub fn into_schema(self) -> DepartmentAccount {
        DepartmentAccount {
            id: self.id,
            display_name: self.display_name.to_string(),

            is_active: self.is_active.to_owned(),
            is_deleted: self.is_deleted,
            verified: self.verified,
        }
    }
}
