use crate::{errors::GenericError, utils::error_chain_fmt};

#[allow(clippy::enum_variant_names)]
#[derive(thiserror::Error)]

pub enum ProjectAccountError {
    #[error("{0}, {1}")]
    DatabaseError(String, anyhow::Error),
    #[error("Invalid Role")]
    InvalidRoleError(String),
    #[error(transparent)]
    UnexpectedError(#[from] anyhow::Error),
}

impl std::fmt::Debug for ProjectAccountError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        error_chain_fmt(self, f)
    }
}

impl From<ProjectAccountError> for GenericError {
    fn from(err: ProjectAccountError) -> GenericError {
        match err {
            ProjectAccountError::UnexpectedError(error) => GenericError::UnexpectedError(error),
            ProjectAccountError::DatabaseError(message, error) => {
                GenericError::DatabaseError(message, error)
            }
            ProjectAccountError::InvalidRoleError(message) => {
                GenericError::ValidationError(message)
            }
        }
    }
}
