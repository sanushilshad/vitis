use crate::errors::GenericError;
use actix_http::StatusCode;
use actix_web::{FromRequest, HttpMessage};
use anyhow::anyhow;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::fmt::{self, Debug};
use std::future::{Ready, ready};
use utoipa::ToSchema;
use uuid::Uuid;
#[derive(Serialize, Debug, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct GenericResponse<D> {
    pub status: bool,
    pub customer_message: String,
    pub code: String,
    pub data: D,
}

impl<D> GenericResponse<D> {
    pub fn success(message: &str, data: D) -> Self {
        Self {
            status: true,
            customer_message: String::from(message),
            code: StatusCode::OK.as_str().to_owned(),
            data,
        }
    }

    pub fn error(message: &str, code: StatusCode, data: D) -> Self {
        Self {
            status: false,
            customer_message: String::from(message),
            code: code.as_str().to_owned(),
            data,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, sqlx::Type, Clone, ToSchema)]
#[sqlx(rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum MaskingType {
    NA,
    Encrypt,
    PartialMask,
    FullMask,
}

#[derive(Serialize, Deserialize, Debug, sqlx::Type, Clone, PartialEq, ToSchema)]
#[sqlx(rename_all = "lowercase", type_name = "status")]
#[serde(rename_all = "lowercase")]
pub enum Status {
    Active,
    Inactive,
    Pending,
    Archived,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct RequestMetaData {
    pub device_id: String,
    pub request_id: String,
}

impl FromRequest for RequestMetaData {
    type Error = GenericError;
    type Future = Ready<Result<Self, Self::Error>>;

    fn from_request(
        req: &actix_web::HttpRequest,
        _payload: &mut actix_web::dev::Payload,
    ) -> Self::Future {
        let value = req.extensions().get::<RequestMetaData>().cloned();

        let result = match value {
            Some(data) => Ok(data),

            None => Err(GenericError::ValidationError(
                "Something went wrong while setting meta data".to_string(),
            )),
        };

        ready(result)
    }
}

#[derive(Deserialize, Debug, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum DeleteType {
    Hard,
    Soft,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct WebSocketParam {
    pub user_id: Option<Uuid>,
    pub business_id: Option<Uuid>,
    pub device_id: Option<String>,
}

#[allow(dead_code)]
pub trait WSKeyTrait {
    fn get_ws_key(&self) -> String;
}

impl WSKeyTrait for WebSocketParam {
    fn get_ws_key(&self) -> String {
        format!(
            "{}#{}#{}",
            self.user_id.map_or("NA".to_string(), |id| id.to_string()),
            self.business_id
                .map_or("NA".to_string(), |id| id.to_string()),
            self.device_id.clone().unwrap_or("NA".to_string())
        )
    }
}

#[derive(Debug, Serialize, Clone, Deserialize, PartialEq)]
pub enum PermissionType {
    #[serde(rename = "create:user-setting")]
    CreateUserSetting,
    #[serde(rename = "create:user-setting:self")]
    CreateUserSettingSelf,
    #[serde(rename = "associate:user-business")]
    AssociateUserBusiness,
    #[serde(rename = "create:leave-request:self")]
    CreateLeaveRequestSelf,
    #[serde(rename = "create:leave-request")]
    CreateLeaveRequest,
    #[serde(rename = "approve:leave-request")]
    ApproveLeaveRequest,
    #[serde(rename = "update:leave-request-status")]
    UpdateLeaveRequestStatus,
    #[serde(rename = "list:users")]
    ListUsers,
    #[serde(rename = "create:global-setting")]
    CreateGlobalSetting,
    #[serde(rename = "create:business-setting")]
    CreateBusinessSetting,
    #[serde(rename = "create:business-setting:self")]
    CreateBusinessSettingSelf,
    #[serde(rename = "associate:user-department")]
    AssociateUserDepartment,
    #[serde(rename = "create:department")]
    CreateDepartment,
    #[serde(rename = "list:leave-request:self")]
    ListLeaveRequestSelf,
    #[serde(rename = "list:leave-request")]
    ListLeaveRequest,
    #[serde(rename = "create:user-business-setting")]
    CreateUserBusinessSetting,
    #[serde(rename = "create:leave-type")]
    CreateLeaveType,
    #[serde(rename = "list:user-business")]
    ListUserBusiness,
    #[serde(rename = "list:user-business:self")]
    ListUserBusinessSelf,
    #[serde(rename = "list:user-leave:self")]
    ListUserLeaveSelf,
    #[serde(rename = "list:user-leave")]
    ListUserLeave,
    #[serde(rename = "send:business-invite")]
    SendBusinessInvite,
    #[serde(rename = "delete:business")]
    DeleteBusiness,
    #[serde(rename = "update:business")]
    UpdateBusiness,
    #[serde(rename = "disassociate:user-business:self")]
    DisassociateBusinessSelf,
    #[serde(rename = "disassociate:user-business")]
    DisassociateBusiness,
}

impl fmt::Display for PermissionType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let display_str = match self {
            PermissionType::CreateUserSetting => "create:user-setting",
            PermissionType::AssociateUserBusiness => "associate:user-business",
            PermissionType::CreateLeaveRequestSelf => "create:leave-request:self",
            PermissionType::CreateLeaveRequest => "create:leave-request",
            PermissionType::ApproveLeaveRequest => "approve:leave-request",
            PermissionType::UpdateLeaveRequestStatus => "update:leave-request-status",
            PermissionType::CreateUserSettingSelf => "create:user-setting:self",
            PermissionType::ListUsers => "list:users",
            PermissionType::CreateGlobalSetting => "create:global-setting",
            PermissionType::AssociateUserDepartment => "associate:user-department",
            PermissionType::CreateBusinessSetting => "create:business-setting",
            PermissionType::CreateBusinessSettingSelf => "create:business-setting:self",
            PermissionType::CreateDepartment => "create:department",
            PermissionType::ListLeaveRequestSelf => "list:leave-request:self",
            PermissionType::ListLeaveRequest => "list:leave-request",
            PermissionType::CreateUserBusinessSetting => "list:leave-request",
            PermissionType::CreateLeaveType => "create:leave-type",
            PermissionType::ListUserBusinessSelf => "list:user-business:self",
            PermissionType::ListUserBusiness => "list:user-business",
            PermissionType::ListUserLeaveSelf => "list:user-leave:self",
            PermissionType::ListUserLeave => "list:user-leave:self",
            PermissionType::SendBusinessInvite => "send:business-invite",
            PermissionType::DeleteBusiness => "delete:business",
            PermissionType::UpdateBusiness => "update:business",
            PermissionType::DisassociateBusinessSelf => "disassociate:user-business:self",
            PermissionType::DisassociateBusiness => "disassociate:user-business",
        };

        write!(f, "{}", display_str)
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct AllowedPermission {
    pub permission_list: Vec<String>,
}

impl AllowedPermission {
    pub fn _is_present(&self, permission: String) -> bool {
        self.permission_list.contains(&permission)
    }
}

impl FromRequest for AllowedPermission {
    type Error = GenericError;
    type Future = Ready<Result<Self, Self::Error>>;

    fn from_request(
        req: &actix_web::HttpRequest,
        _payload: &mut actix_web::dev::Payload,
    ) -> Self::Future {
        let value = req.extensions().get::<AllowedPermission>().cloned();

        let result = match value {
            Some(user) => Ok(user),
            None => Err(GenericError::UnexpectedError(anyhow!(
                "Something went wrong while parsing allowed_permission data".to_string()
            ))),
        };

        ready(result)
    }
}

#[derive(Debug, Deserialize, sqlx::Type)]
#[sqlx(type_name = "alert_status", rename_all = "lowercase")]
pub enum AlertStatus {
    Pending,
    Success,
    Failed,
}

#[derive(Debug)]
pub struct BulkNotificationData<'a> {
    pub id: Vec<Uuid>,
    pub data: Vec<&'a Value>,
    pub connection_id: Vec<&'a str>,
    pub created_on: Vec<&'a DateTime<Utc>>,
}
