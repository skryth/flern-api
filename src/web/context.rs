//! Request context, e.g. user id, its role, etc.
//!

use axum::{extract::FromRequestParts, http::request::Parts};

use crate::web::{WebResult, error::WebError};

#[derive(Debug, Clone)]
pub struct AuthenticatedUser {
    user_id: uuid::Uuid,
    user_role: UserRole,
}

impl AuthenticatedUser {
    pub fn new(user_id: uuid::Uuid, user_role: UserRole) -> Self {
        Self { user_id, user_role }
    }

    pub fn admin() -> Self {
        Self {
            user_role: UserRole::Admin,
            user_id: uuid::Uuid::max(), // admin ID
        }
    }

    pub fn user_id(&self) -> uuid::Uuid {
        self.user_id
    }

    pub fn user_role(&self) -> UserRole {
        self.user_role.clone()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum UserRole {
    Admin,
    User,
}

impl From<&str> for UserRole {
    fn from(value: &str) -> Self {
        match value {
            "admin" => Self::Admin,
            _ => Self::User,
        }
    }
}

impl std::fmt::Display for UserRole {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::User => write!(f, "user"),
            Self::Admin => write!(f, "admin"),
        }
    }
}

#[derive(Debug, Clone)]
pub struct RequestContext {
    maybe_user: Option<AuthenticatedUser>,
}

impl RequestContext {
    pub fn new(maybe_user: Option<AuthenticatedUser>) -> Self {
        Self { maybe_user }
    }

    pub fn admin() -> Self {
        Self::new(Some(AuthenticatedUser::admin()))
    }

    pub fn maybe_user(&self) -> Option<&AuthenticatedUser> {
        self.maybe_user.as_ref()
    }

    pub fn user(&self) -> WebResult<&AuthenticatedUser> {
        self.maybe_user.as_ref().ok_or(WebError::auth_required())
    }
}

impl<S> FromRequestParts<S> for RequestContext
where
    S: Send + Sync,
{
    type Rejection = WebError;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        let ctx = parts.extensions.get::<RequestContext>();
        if let Some(ctx) = ctx {
            Ok(ctx.clone())
        } else {
            Ok(RequestContext::new(None))
        }
    }
}
