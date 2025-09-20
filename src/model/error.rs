use thiserror::Error;

pub type DatabaseResult<T> = std::result::Result<T, DatabaseError>;

#[derive(Debug, Error)]
pub enum DatabaseError {
    #[error("sqlx migrate error: {0}")]
    SqlxMigrateError(#[from] sqlx::migrate::MigrateError),
    #[error("sqlx error: {0}")]
    SqlxError(#[from] sqlx::Error),
    #[error("json error: {0}")]
    SerdeError(#[from] serde_json::Error),
    #[error("access to this resource is forbidden")]
    Forbidden,
}
