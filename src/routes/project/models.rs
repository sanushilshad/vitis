use sqlx::{FromRow, types::Json};
use uuid::Uuid;

use crate::{routes::user::schemas::UserVector, schemas::Status};

use super::schemas::{BasicprojectAccount, ProjectAccount};

#[allow(dead_code)]
#[derive(Debug, FromRow)]
pub struct ProjectAccountModel {
    pub id: Uuid,
    pub name: String,
    pub vectors: Json<Vec<UserVector>>,
    pub is_active: Status,
    pub is_deleted: bool,
    pub verified: bool,
}

impl ProjectAccountModel {
    pub fn into_basic_schema(self) -> BasicprojectAccount {
        BasicprojectAccount {
            id: self.id,
            name: self.name,
        }
    }
    pub fn into_schema(self) -> ProjectAccount {
        ProjectAccount {
            id: self.id,
            name: self.name.to_string(),
            vectors: self.vectors.0.to_owned(),
            is_active: self.is_active.to_owned(),
            is_deleted: self.is_deleted,
            verified: self.verified,
        }
    }
}

#[allow(dead_code)]
#[derive(Debug, FromRow)]
pub struct UserprojectRelationAccountModel {
    pub id: Uuid,
    pub name: String,
    pub vectors: Json<Vec<UserVector>>,
    pub is_active: Status,
    pub verified: bool,
    pub is_deleted: bool,
}

impl UserprojectRelationAccountModel {
    pub fn into_schema(self) -> ProjectAccount {
        ProjectAccount {
            id: self.id,
            name: self.name.to_string(),
            vectors: self.vectors.0.to_owned(),

            is_active: self.is_active.to_owned(),
            is_deleted: self.is_deleted,
            verified: self.verified,
        }
    }
}
