mod context;
pub use context::{AuthenticatedUser, RequestContext, UserRole};

mod error;
pub use error::{WebError, WebResult};

pub mod middlewares;

mod state;
pub use state::AppState;

pub mod routes;

pub mod doc;
