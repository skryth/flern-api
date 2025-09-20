use axum::{Json, http::StatusCode, response::IntoResponse};
use thiserror::Error;

use crate::{
    auth::CryptError,
    error::log_error,
    model::{DatabaseError, ResourceType},
};

pub type WebResult<T> = std::result::Result<T, WebError>;

#[derive(Debug, Error)]
pub enum RegistrationError {
    #[error("RegistrationUserConflict")]
    RegistrationUserConflict,
}

#[derive(Debug, Error)]
pub enum AuthenticationError {
    #[error("AuthenticationCookieNotFound, cookie: {cookie}")]
    AuthenticationCookieNotFound { cookie: String },

    #[error("AuthenticationCookieInvalid, cookie: {cookie}. Error: {error}")]
    AuthenticationCookieInvalid {
        cookie: String,
        error: jsonwebtoken::errors::Error,
    },

    #[error("AuthenticationRequired")]
    AuthenticationRequired,

    #[error("AuthenticationInvalidCredentials")]
    AuthenticationInvalidCredentials,
}

#[derive(Debug, Error)]
pub enum ResourceError {
    #[error("ResourceNotFound: {resource_type:?}")]
    ResourceNotFound { resource_type: ResourceType },

    #[error("ResourceForbidden: {resource_type:?}")]
    ResourceForbidden { resource_type: ResourceType },

    #[error("ResourceFetchError: {resource_type:?}. Error: {error}")]
    ResourceFetchError {
        resource_type: ResourceType,
        error: DatabaseError,
    },

    #[error("ResourceBadRequest: {resource_type:?}")]
    ResourceBadRequest {
        resource_type: ResourceType,
        // TODO: Maybe some string of details
    },
}

#[derive(Debug, Error)]
pub enum ServerError {
    #[error("ServerCryptError: {0}")]
    ServerCryptError(#[from] crate::auth::CryptError),
}

impl ServerError {
    pub fn status_code(&self) -> StatusCode {
        StatusCode::INTERNAL_SERVER_ERROR
    }

    pub fn client_display(&self) -> String {
        String::from("Internal server error.")
    }
}

impl RegistrationError {
    pub fn status_code(&self) -> StatusCode {
        match self {
            Self::RegistrationUserConflict => StatusCode::CONFLICT,
        }
    }

    pub fn client_display(&self) -> String {
        match self {
            Self::RegistrationUserConflict => {
                String::from("Registration error, user already exists.")
            }
        }
    }
}

impl AuthenticationError {
    pub fn status_code(&self) -> StatusCode {
        match self {
            Self::AuthenticationRequired => StatusCode::UNAUTHORIZED,
            Self::AuthenticationCookieNotFound { .. } => StatusCode::NOT_FOUND,
            Self::AuthenticationInvalidCredentials => StatusCode::UNAUTHORIZED,
            Self::AuthenticationCookieInvalid { .. } => StatusCode::BAD_REQUEST,
        }
    }

    pub fn client_display(&self) -> String {
        match self {
            Self::AuthenticationCookieInvalid { .. } => {
                String::from("Authentication error, cookie invalid.")
            }
            Self::AuthenticationCookieNotFound { .. } => {
                String::from("Authentication error, cookie not found.")
            }
            Self::AuthenticationRequired => String::from("Authentication required."),
            Self::AuthenticationInvalidCredentials => {
                String::from("Authentication error, user not found or password is invalid.")
            }
        }
    }
}

impl ResourceError {
    pub fn status_code(&self) -> StatusCode {
        match self {
            Self::ResourceNotFound { .. } => StatusCode::NOT_FOUND,
            Self::ResourceForbidden { .. } => StatusCode::FORBIDDEN,
            Self::ResourceFetchError { .. } => StatusCode::INTERNAL_SERVER_ERROR,
            Self::ResourceBadRequest { .. } => StatusCode::BAD_REQUEST,
        }
    }

    pub fn client_display(&self) -> String {
        match self {
            Self::ResourceNotFound { .. } => String::from("Resource error, resource not found."),
            Self::ResourceForbidden { .. } => String::from("Resource error, resource forbidden."),
            Self::ResourceFetchError { .. } => {
                String::from("Resource error, unable to fetch resource.")
            }
            Self::ResourceBadRequest { .. } => String::from("Resource error, bad request."),
        }
    }
}

#[derive(Debug, Error)]
pub enum WebError {
    #[error("ResourceError - {0}")]
    ResourceError(#[from] ResourceError),
    #[error("AuthenticationError - {0}")]
    AuthenticationError(#[from] AuthenticationError),
    #[error("RegistrationError - {0}")]
    RegistrationError(#[from] RegistrationError),
    #[error("ServerError - {0}")]
    ServerError(#[from] ServerError),
}

impl WebError {
    pub fn resource_not_found(r#type: ResourceType) -> Self {
        Self::ResourceError(ResourceError::ResourceNotFound {
            resource_type: r#type,
        })
    }

    pub fn resource_forbidden(r#type: ResourceType) -> Self {
        Self::ResourceError(ResourceError::ResourceForbidden {
            resource_type: r#type,
        })
    }

    pub fn resource_fetch_error(r#type: ResourceType, error: DatabaseError) -> Self {
        Self::ResourceError(ResourceError::ResourceFetchError {
            resource_type: r#type,
            error,
        })
    }

    pub fn resource_bad_request(r#type: ResourceType) -> Self {
        Self::ResourceError(ResourceError::ResourceBadRequest {
            resource_type: r#type,
        })
    }

    pub fn auth_cookie_not_found<S: Into<String>>(cookie: S) -> Self {
        Self::AuthenticationError(AuthenticationError::AuthenticationCookieNotFound {
            cookie: cookie.into(),
        })
    }

    pub fn auth_cookie_invalid<S: Into<String>>(
        cookie: S,
        error: jsonwebtoken::errors::Error,
    ) -> Self {
        Self::AuthenticationError(AuthenticationError::AuthenticationCookieInvalid {
            cookie: cookie.into(),
            error,
        })
    }

    pub fn auth_required() -> Self {
        Self::AuthenticationError(AuthenticationError::AuthenticationRequired)
    }

    pub fn auth_invalid_credentials() -> Self {
        Self::AuthenticationError(AuthenticationError::AuthenticationInvalidCredentials)
    }

    pub fn registration_conflict() -> Self {
        Self::RegistrationError(RegistrationError::RegistrationUserConflict)
    }

    pub fn server_crypt_error(e: CryptError) -> Self {
        Self::ServerError(ServerError::ServerCryptError(e))
    }

    pub fn status_code(&self) -> axum::http::StatusCode {
        match self {
            Self::ResourceError(e) => e.status_code(),
            Self::RegistrationError(e) => e.status_code(),
            Self::AuthenticationError(e) => e.status_code(),
            Self::ServerError(e) => e.status_code(),
        }
    }

    pub fn client_display(&self) -> String {
        match self {
            Self::ResourceError(e) => e.client_display(),
            Self::RegistrationError(e) => e.client_display(),
            Self::AuthenticationError(e) => e.client_display(),
            Self::ServerError(e) => e.client_display(),
        }
    }
}

#[derive(serde::Serialize, utoipa::ToSchema)]
pub struct ErrorResponse {
    /// Human-readable message for the client
    pub message: String,
    /// HTTP status code (stringified)
    pub status_code: String,
    /// Optional debug details (only in debug mode)
    pub details: Option<String>,
}

impl IntoResponse for WebError {
    fn into_response(self) -> axum::response::Response {
        log_error(&self);

        let status_code = self.status_code();
        let display = self.client_display();

        let body = ErrorResponse {
            message: display,
            status_code: status_code.as_str().to_string(),
            details: if cfg!(debug_assertions) {
                Some(self.to_string())
            } else {
                None
            },
        };

        (status_code, Json(body)).into_response()
    }
}
