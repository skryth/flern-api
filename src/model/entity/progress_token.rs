use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::prelude::FromRow;
use uuid::Uuid;

use crate::{model::{CrudRepository, DatabaseResult, ModelManager, ResourceTyped}, web::AuthenticatedUser};


#[derive(Debug, Serialize, Deserialize, FromRow, utoipa::ToSchema)]
pub struct ProgressToken {
    id: Uuid,
    token: String,
    user_id: Uuid,
    expires_at: DateTime<Utc>,
    created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ProgressTokenCreate {
    pub token: String,
    pub user_id: Uuid,
    pub expires_at: DateTime<Utc>,
}

impl ResourceTyped for ProgressToken {
    fn get_resource_type() -> crate::model::ResourceType {
        crate::model::ResourceType::ProgressToken
    }
}

impl ProgressToken {
    pub fn id(&self) -> Uuid {
        self.id
    }

    pub fn token(&self) -> &str {
        &self.token
    }

    pub fn user_id(&self) -> Uuid {
        self.user_id
    }

    pub fn expires_at(&self) -> &DateTime<Utc> {
        &self.expires_at
    }

    pub fn created_at(&self) -> &DateTime<Utc> {
        &self.created_at
    }
}

#[async_trait]
impl CrudRepository<ProgressToken, ProgressTokenCreate, uuid::Uuid> for ProgressToken {
    async fn create(
        mm: &ModelManager,
        _actor: &AuthenticatedUser,
        data: ProgressTokenCreate,
    ) -> DatabaseResult<Self> {
        let result = sqlx::query_as("INSERT INTO progress_tokens (id, token, user_id, expires_at) VALUES ($1,$2,$3,$4) RETURNING id, token, user_id, expires_at, created_at")
            .bind(Uuid::new_v4())
            .bind(data.token)
            .bind(data.user_id)
            .bind(data.expires_at)
            .fetch_one(mm.executor())
            .await?;

        Ok(result)
    }

    async fn update(
        mut self,
        _mm: &ModelManager,
        _actor: &AuthenticatedUser,
        _data: ProgressTokenCreate,
    ) -> DatabaseResult<Self> {
        unimplemented!("Progress tokens should never be updated");
    }

    async fn delete(self, mm: &ModelManager, _actor: &AuthenticatedUser) -> DatabaseResult<()> {
        sqlx::query("DELETE FROM progress_tokens WHERE id = $1")
            .bind(self.id)
            .execute(mm.executor())
            .await?;
        Ok(())
    }

    async fn find_by_id(
        mm: &ModelManager,
        _actor: &AuthenticatedUser,
        id: uuid::Uuid,
    ) -> DatabaseResult<Option<Self>> {
        let result = sqlx::query_as("SELECT * FROM progress_tokens WHERE id = $1")
            .bind(id)
            .fetch_one(mm.executor())
            .await;
        if let Err(sqlx::Error::RowNotFound) = result {
            return Ok(None);
        }

        Ok(Some(result?))
    }

    async fn list(
        mm: &ModelManager,
        _actor: &AuthenticatedUser,
        limit: i64,
        offset: i64,
    ) -> DatabaseResult<Vec<Self>> {
        let result = sqlx::query_as("SELECT * FROM progress_tokens LIMIT $1 OFFSET $2")
            .bind(limit)
            .bind(offset)
            .fetch_all(mm.executor())
            .await?;
        Ok(result)
    }

    async fn count(mm: &ModelManager, _actor: &AuthenticatedUser) -> DatabaseResult<i64> {
        let result: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM progress_tokens")
            .fetch_one(mm.executor())
            .await?;

        Ok(result)
    }
}

impl ProgressToken {
    pub async fn find_by_token(
        mm: &ModelManager,
        _actor: &AuthenticatedUser,
        token: &str
    ) -> DatabaseResult<Option<Self>> {
        let result = sqlx::query_as("SELECT * FROM progress_tokens WHERE token = $1")
            .bind(token)
            .fetch_optional(mm.executor())
            .await?;

        Ok(result)
    }

    pub async fn cleanup_expired(
        mm: &ModelManager,
        _actor: &AuthenticatedUser,
    ) -> DatabaseResult<u64> {
        let result = sqlx::query(r#"DELETE FROM progress_tokens WHERE expires_at < now()"#)
            .execute(mm.executor())
            .await?;

        Ok(result.rows_affected())
    }
}
