use sqlx::{FromRow, types::Json};
use uuid::Uuid;

use crate::{routes::user::schemas::UserVector, schemas::Status};

use super::schemas::{BasicBusinessAccount, BusinessAccount};

#[derive(Debug, FromRow)]
pub struct BusinessAccountModel {
    pub id: Uuid,
    pub display_name: String,
    pub vectors: Json<Vec<UserVector>>,
    pub is_active: Status,
    pub is_deleted: bool,
    pub verified: bool,
}

impl BusinessAccountModel {
    pub fn into_basic_schema(self) -> BasicBusinessAccount {
        BasicBusinessAccount {
            id: self.id,
            display_name: self.display_name,
        }
    }
    pub fn into_schema(self) -> BusinessAccount {
        BusinessAccount {
            id: self.id,
            display_name: self.display_name.to_string(),
            vectors: self.vectors.0.to_owned(),
            is_active: self.is_active.to_owned(),
            is_deleted: self.is_deleted,
            verified: self.verified,
        }
    }
}

#[derive(Debug, FromRow)]
pub struct UserBusinessRelationAccountModel {
    pub id: Uuid,
    pub display_name: String,
    pub vectors: Json<Vec<UserVector>>,
    pub is_active: Status,
    pub verified: bool,
    pub is_deleted: bool,
}

impl UserBusinessRelationAccountModel {
    pub fn into_schema(self) -> BusinessAccount {
        BusinessAccount {
            id: self.id,
            display_name: self.display_name.to_string(),
            vectors: self.vectors.0.to_owned(),

            is_active: self.is_active.to_owned(),
            is_deleted: self.is_deleted,
            verified: self.verified,
        }
    }
}
