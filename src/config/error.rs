use thiserror::Error;

pub type ConfigResult<T> = std::result::Result<T, ConfigError>;

#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("io error: {0}")]
    IoError(#[from] std::io::Error),
    #[error("de error: {0}")]
    TomlDeError(#[from] toml::de::Error),
    #[error("se error: {0}")]
    TomlSeError(#[from] toml::ser::Error),
    #[error("config not found")]
    ConfigNotFound,
}
