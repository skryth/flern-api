use sqlx::PgPool;
use crate::model::error::DatabaseResult;

#[derive(Debug, Clone)]
pub struct DbConnection {
    pool: PgPool, // cloning is cheap, pool is just a wrapper around Arc<>
}

impl DbConnection {
    pub fn connect(connection_str: &str) -> DatabaseResult<Self> {
        let pool = PgPool::connect_lazy(connection_str)?;
        Ok(Self { pool })
    }

    pub fn pool(&self) -> &PgPool {
        &self.pool
    }

    pub fn from_pool(pool: PgPool) -> Self {
        Self { pool }
    }
}
