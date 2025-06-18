use crate::{errors::GenericError, utils::error_chain_fmt};

#[allow(clippy::enum_variant_names)]
#[derive(thiserror::Error)]

pub enum BusinessAccountError {
    #[error("{0}, {1}")]
    DatabaseError(String, anyhow::Error),
    #[error("Invalid Role")]
    InvalidRoleError(String),
    #[error(transparent)]
    UnexpectedError(#[from] anyhow::Error),
}

impl std::fmt::Debug for BusinessAccountError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        error_chain_fmt(self, f)
    }
}

impl From<BusinessAccountError> for GenericError {
    fn from(err: BusinessAccountError) -> GenericError {
        match err {
            BusinessAccountError::UnexpectedError(error) => GenericError::UnexpectedError(error),
            BusinessAccountError::DatabaseError(message, error) => {
                GenericError::DatabaseError(message, error)
            }
            BusinessAccountError::InvalidRoleError(message) => {
                GenericError::ValidationError(message)
            }
        }
    }
}
