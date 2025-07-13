use crate::{errors::GenericError, utils::error_chain_fmt};

#[derive(thiserror::Error)]
#[allow(dead_code)]
pub enum UserRegistrationError {
    #[error("Duplicate email")]
    DuplicateEmail(String),
    #[error(transparent)]
    UnexpectedError(#[from] anyhow::Error),
    #[error("Duplicate mobile no")]
    DuplicateMobileNo(String),
    #[error("{0}")]
    DatabaseError(String, anyhow::Error),
    #[error("Insufficient previlege to register Admin/Superadmin")]
    InsufficientPrevilegeError(String),
    #[error("Invalid Role")]
    InvalidRoleError(String),
}

impl std::fmt::Debug for UserRegistrationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        error_chain_fmt(self, f)
    }
}

impl From<UserRegistrationError> for GenericError {
    fn from(err: UserRegistrationError) -> GenericError {
        match err {
            UserRegistrationError::DuplicateEmail(message) => {
                GenericError::ValidationError(message)
            }
            UserRegistrationError::DuplicateMobileNo(message) => {
                GenericError::ValidationError(message)
            }
            UserRegistrationError::UnexpectedError(error) => GenericError::UnexpectedError(error),
            UserRegistrationError::DatabaseError(message, error) => {
                GenericError::DatabaseError(message, error)
            }
            UserRegistrationError::InvalidRoleError(message) => {
                GenericError::ValidationError(message)
            }
            UserRegistrationError::InsufficientPrevilegeError(message) => {
                GenericError::InsufficientPrevilegeError(message)
            }
        }
    }
}

#[derive(thiserror::Error)]
#[allow(dead_code)]
pub enum AuthError {
    #[error("{0}")]
    InvalidCredentials(String),
    #[error(transparent)]
    UnexpectedError(#[from] anyhow::Error),
    #[error("{0}")]
    UnexpectedCustomError(String),
    #[error("{0}")]
    DatabaseError(String, anyhow::Error),
    #[error("{0}")]
    TooManyRequest(String),
}

impl std::fmt::Debug for AuthError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        error_chain_fmt(self, f)
    }
}

impl From<AuthError> for GenericError {
    fn from(err: AuthError) -> GenericError {
        match err {
            AuthError::UnexpectedError(error) => GenericError::UnexpectedError(error),
            AuthError::DatabaseError(message, error) => GenericError::DatabaseError(message, error),
            AuthError::InvalidCredentials(message) => GenericError::UnAuthorized(message),
            AuthError::UnexpectedCustomError(message) => GenericError::ValidationError(message),
            AuthError::TooManyRequest(message) => GenericError::TooManyRequest(message),
        }
    }
}
