use crate::{errors::GenericError, utils::error_chain_fmt};

#[allow(clippy::enum_variant_names)]
#[derive(thiserror::Error)]
#[allow(dead_code)]
pub enum DepartmentAccountError {
    #[error("{0}, {1}")]
    DatabaseError(String, anyhow::Error),
    #[error("Invalid Role")]
    InvalidRoleError(String),
    #[error(transparent)]
    UnexpectedError(#[from] anyhow::Error),
}

impl std::fmt::Debug for DepartmentAccountError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        error_chain_fmt(self, f)
    }
}

impl From<DepartmentAccountError> for GenericError {
    fn from(err: DepartmentAccountError) -> GenericError {
        match err {
            DepartmentAccountError::UnexpectedError(error) => GenericError::UnexpectedError(error),
            DepartmentAccountError::DatabaseError(message, error) => {
                GenericError::DatabaseError(message, error)
            }
            DepartmentAccountError::InvalidRoleError(message) => {
                GenericError::ValidationError(message)
            }
        }
    }
}
