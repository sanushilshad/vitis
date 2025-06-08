use actix_http::StatusCode;
use actix_web::{HttpResponse, ResponseError};

use crate::{schemas::GenericResponse, utils::error_chain_fmt};

#[derive(thiserror::Error)]
pub enum CustomJWTTokenError {
    #[error("Token expired")]
    Expired,
    #[error("{0}")]
    Invalid(String),
}

impl std::fmt::Debug for CustomJWTTokenError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        error_chain_fmt(self, f)
    }
}

#[allow(dead_code)]
#[derive(thiserror::Error)]
pub enum GenericError {
    #[error("{0}")]
    ValidationError(String),
    #[error(transparent)]
    UnexpectedError(#[from] anyhow::Error),
    #[error("{0}")]
    DatabaseError(String, anyhow::Error),
    #[error("{0}")]
    SerializationError(String),
    #[error("Insufficient previlege")]
    InsufficientPrevilegeError(String),
    #[error("{0}")]
    UnAuthorized(String),
    #[error("{0}")]
    UnexpectedCustomError(String),
    #[error("{0}")]
    InvalidData(String),
    #[error("{0}")]
    DataNotFound(String),
    #[error("{0}")]
    DataAlreadyExist(String),
    #[error("{0}")]
    TooManyRequest(String),
}

impl std::fmt::Debug for GenericError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        error_chain_fmt(self, f)
    }
}

impl ResponseError for GenericError {
    fn status_code(&self) -> StatusCode {
        match self {
            GenericError::ValidationError(_) => StatusCode::BAD_REQUEST,
            GenericError::UnexpectedError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            GenericError::DatabaseError(_, _) => StatusCode::INTERNAL_SERVER_ERROR,
            GenericError::SerializationError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            GenericError::InsufficientPrevilegeError(_) => StatusCode::FORBIDDEN,
            GenericError::UnAuthorized(_) => StatusCode::UNAUTHORIZED,
            GenericError::UnexpectedCustomError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            GenericError::InvalidData(_) => StatusCode::INTERNAL_SERVER_ERROR,
            GenericError::DataNotFound(_) => StatusCode::GONE,
            GenericError::DataAlreadyExist(_) => StatusCode::CONFLICT,
            GenericError::TooManyRequest(_) => StatusCode::TOO_MANY_REQUESTS,
        }
    }

    fn error_response(&self) -> HttpResponse {
        let status_code = self.status_code();
        let inner_error_msg = match self {
            GenericError::ValidationError(message) => message.to_string(),
            GenericError::UnexpectedError(error_msg) => error_msg.to_string(),
            GenericError::DatabaseError(message, _err) => message.to_string(),
            GenericError::SerializationError(message) => message.to_string(),
            GenericError::InsufficientPrevilegeError(error_msg) => error_msg.to_string(),
            GenericError::UnAuthorized(error_msg) => error_msg.to_string(),
            GenericError::UnexpectedCustomError(error_msg) => error_msg.to_string(),
            GenericError::InvalidData(error_msg) => error_msg.to_string(),
            GenericError::DataNotFound(error_msg) => error_msg.to_string(),
            GenericError::DataAlreadyExist(error_msg) => error_msg.to_string(),
            GenericError::TooManyRequest(error_msg) => error_msg.to_string(),
        };

        HttpResponse::build(status_code).json(GenericResponse::error(
            &inner_error_msg,
            status_code,
            (),
        ))
    }
}

impl From<serde_json::Error> for GenericError {
    fn from(error: serde_json::Error) -> Self {
        GenericError::UnexpectedError(anyhow::Error::new(error))
    }
}
