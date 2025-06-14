use sqlx::{FromRow, types::Json};
use uuid::Uuid;

use crate::{routes::user::schemas::UserVector, schemas::Status};

use super::schemas::{BasicDepartmentAccount, DepartmentAccount};

#[derive(Debug, FromRow)]
pub struct DepartmentAccountModel {
    pub id: Uuid,
    pub name: String,
    pub is_active: Status,
    pub is_deleted: bool,
    pub verified: bool,
}

impl DepartmentAccountModel {
    pub fn into_basic_schema(self) -> BasicDepartmentAccount {
        BasicDepartmentAccount {
            id: self.id,
            name: self.name,
        }
    }
    pub fn into_schema(self) -> DepartmentAccount {
        DepartmentAccount {
            id: self.id,
            name: self.name.to_string(),
            is_active: self.is_active.to_owned(),
            is_deleted: self.is_deleted,
            verified: self.verified,
        }
    }
}

#[derive(Debug, FromRow)]
pub struct UserDepartmentRelationAccountModel {
    pub id: Uuid,
    pub name: String,
    pub is_active: Status,
    pub verified: bool,
    pub is_deleted: bool,
}

impl UserDepartmentRelationAccountModel {
    pub fn into_schema(self) -> DepartmentAccount {
        DepartmentAccount {
            id: self.id,
            name: self.name.to_string(),

            is_active: self.is_active.to_owned(),
            is_deleted: self.is_deleted,
            verified: self.verified,
        }
    }
}
