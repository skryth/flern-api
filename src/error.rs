use thiserror::Error;
use tracing::error;
use tracing_error::SpanTrace;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("io error: {0}")]
    IoError(#[from] std::io::Error),
    #[error("config error: {0}")]
    ConfigError(#[from] crate::config::ConfigError),
    #[error("database error: {0}")]
    DatabaseError(#[from] crate::model::DatabaseError),
}

pub type AppResult<T> = std::result::Result<T, AppError>;

pub async fn run_with_error_handler<F, T>(run: F) -> T
where
    F: AsyncFn() -> AppResult<T>,
    T: Send + Sync,
{
    let result = run().await;

    if let Err(e) = result {
        default_error_handler(e);
        std::process::exit(1);
    }

    result.unwrap()
}

fn default_error_handler(error: AppError) {
    let span = SpanTrace::capture();
    error!("{}\n{}", error, span);
}

pub fn log_error<E: std::error::Error + std::fmt::Display>(error: &E) {
    let span = SpanTrace::capture();
    error!("{}\n{}", error, span);
}
