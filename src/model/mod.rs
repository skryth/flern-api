mod access;
pub use access::check_access;

mod database;
pub use database::DbConnection;

pub mod entity;

mod error;
pub use error::{DatabaseError, DatabaseResult};

mod repo;
pub use repo::{CrudRepository, Page, PaginatableRepository, ResourceType, ResourceTyped};

use sqlx::PgPool;

#[derive(Debug, Clone)]
pub struct ModelManager {
    database: DbConnection,
}

impl ModelManager {
    pub fn new(conn: DbConnection) -> Self {
        Self { database: conn }
    }

    pub fn executor(&self) -> &PgPool {
        self.database.pool()
    }
}
