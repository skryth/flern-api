use thiserror::Error;

pub type CryptResult<T> = std::result::Result<T, CryptError>;

#[derive(Debug, Error)]
pub enum CryptError {
    #[error("argon2 error: {0}")]
    Argon2Error(#[from] argon2::password_hash::Error),
    #[error("jwt error: {0}")]
    JwtError(#[from] jsonwebtoken::errors::Error),
}
