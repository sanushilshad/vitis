use chrono::{DateTime, Utc};
use secrecy::SecretString;
use sqlx::{FromRow, types::Json};
use uuid::Uuid;

use super::schemas::{
    AccountRole, AuthMechanism, AuthenticationScope, MinimalUserAccount, UserAccount, UserVector,
};
use crate::{email::EmailObject, schemas::Status};
#[derive(Debug, FromRow)]
pub struct AuthMechanismModel {
    pub id: Uuid,
    pub user_id: Uuid,
    pub auth_scope: AuthenticationScope,
    pub auth_identifier: String,
    pub secret: Option<String>,
    pub is_active: Status,
    pub valid_upto: Option<DateTime<Utc>>,
    pub retry_count: Option<i32>, // pub auth_context: AuthContextType,
}

impl AuthMechanismModel {
    pub fn get_schema(self) -> AuthMechanism {
        let secret_string: Option<String> = self.secret;
        let secret = secret_string.map(SecretString::from);
        AuthMechanism {
            id: self.id,
            user_id: self.user_id,
            auth_scope: self.auth_scope,
            auth_identifier: self.auth_identifier,
            secret,
            is_active: self.is_active,
            valid_upto: self.valid_upto,
            retry_count: self.retry_count, // auth_context: self.auth_context,
        }
    }
}

#[derive(Debug, FromRow)]
pub struct UserAccountModel {
    pub id: Uuid,
    pub username: String,
    pub mobile_no: String,
    pub email: String,
    pub is_active: Status,
    pub display_name: String,
    pub vectors: Json<Vec<UserVector>>,
    pub international_dialing_code: String,
    pub is_test_user: bool,
    pub is_deleted: bool,
    pub role_name: String,
}

impl UserAccountModel {
    pub fn into_schema(self) -> UserAccount {
        let vectors_option: Vec<UserVector> = self.vectors.0;
        UserAccount {
            id: self.id,
            mobile_no: self.mobile_no,
            username: self.username,
            email: EmailObject::new(self.email),
            is_active: self.is_active,
            display_name: self.display_name,
            vectors: vectors_option,
            international_dialing_code: self.international_dialing_code,
            is_test_user: self.is_test_user,
            is_deleted: self.is_deleted,
            user_role: self.role_name,
        }
    }
}

#[allow(dead_code)]
#[derive(Debug, FromRow)]
pub struct UserRoleModel {
    pub id: Uuid,
    pub role_name: String,
    pub role_status: Status,
    pub created_on: DateTime<Utc>,
    pub created_by: String,
    pub is_deleted: bool,
}

impl UserRoleModel {
    pub fn int_schema(self) -> AccountRole {
        AccountRole {
            id: self.id,
            role_name: self.role_name,
            role_status: self.role_status,
            is_deleted: self.is_deleted,
        }
    }
}

#[derive(Debug, FromRow)]
pub struct MinimalUserAccountModel {
    pub id: Uuid,
    pub mobile_no: String,
    pub display_name: String,
}

impl MinimalUserAccountModel {
    pub fn into_schema(self) -> MinimalUserAccount {
        MinimalUserAccount {
            id: self.id,
            mobile_no: self.mobile_no,
            display_name: self.display_name,
        }
    }
}
