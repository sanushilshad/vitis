use chrono::{DateTime, Utc};
use sqlx::{FromRow, types::Json};
use uuid::Uuid;

use crate::{email::EmailObject, routes::user::schemas::UserVector, schemas::Status};

use super::schemas::{BasicBusinessAccount, BusinessAccount, UserBusinessInvitation};

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

#[derive(Debug, FromRow)]
pub struct UserBusinessInvitationModel {
    pub id: Uuid,
    pub email: String,
    pub business_id: Uuid,
    pub role_id: Uuid,
    pub created_on: DateTime<Utc>,
    pub created_by: Uuid,
    pub verified: bool,
}

impl UserBusinessInvitationModel {
    pub fn into_schema(self) -> UserBusinessInvitation {
        UserBusinessInvitation {
            id: self.id,
            email: EmailObject::new(self.email),
            business_id: self.business_id,
            role_id: self.role_id,
            created_on: self.created_on,
            created_by: self.created_by,
            verified: self.verified,
        }
    }
}
