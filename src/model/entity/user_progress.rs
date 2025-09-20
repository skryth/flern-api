use crate::impl_paginatable_for;
use crate::model::access::HasOwner;
use crate::model::repo::ResourceTyped;
use crate::model::{ModelManager, error::DatabaseResult, repo::CrudRepository};
use crate::web::AuthenticatedUser;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use sqlx::prelude::FromRow;
use sqlx::prelude::Row;
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, FromRow, utoipa::ToSchema)]
pub struct UserProgress {
    id: Uuid,
    user_id: Uuid,
    lesson_id: Uuid,
    status: String,
}

impl ResourceTyped for UserProgress {
    fn get_resource_type() -> crate::model::ResourceType {
        crate::model::ResourceType::UserProgress
    }
}

impl UserProgress {
    pub fn id(&self) -> Uuid {
        self.id
    }

    pub fn user_id(&self) -> Uuid {
        self.user_id
    }

    pub fn lesson_id(&self) -> Uuid {
        self.lesson_id
    }

    pub fn status(&self) -> &str {
        &self.status
    }
}

pub struct UserProgressCreate {
    user_id: Uuid,
    lesson_id: Uuid,
    status: String,
}

#[async_trait]
impl CrudRepository<UserProgress, UserProgressCreate, uuid::Uuid> for UserProgress {
    async fn create(
        mm: &ModelManager,
        _actor: &AuthenticatedUser,
        data: UserProgressCreate,
    ) -> DatabaseResult<Self> {
        let result = sqlx::query("INSERT INTO user_progress (id, user_id, lesson_id, status) VALUES ($1,$2,$3,$4) RETURNING id")
            .bind(Uuid::new_v4())
            .bind(data.user_id)
            .bind(data.lesson_id)
            .bind(&data.status)
            .fetch_one(mm.executor())
            .await?;

        let id = result.try_get("id")?;
        Ok(UserProgress {
            id,
            user_id: data.user_id,
            lesson_id: data.lesson_id,
            status: data.status,
        })
    }

    async fn update(
        mut self,
        mm: &ModelManager,
        _actor: &AuthenticatedUser,
        data: UserProgressCreate,
    ) -> DatabaseResult<Self> {
        sqlx::query(
            "UPDATE user_progress SET user_id = $1, lesson_id = $2, status = $3 WHERE id = $4",
        )
        .bind(data.user_id)
        .bind(data.lesson_id)
        .bind(&data.status)
        .bind(self.id)
        .execute(mm.executor())
        .await?;

        self.user_id = data.user_id;
        self.lesson_id = data.lesson_id;
        self.status = data.status;
        Ok(self)
    }

    async fn delete(self, mm: &ModelManager, _actor: &AuthenticatedUser) -> DatabaseResult<()> {
        sqlx::query("DELETE FROM user_progress WHERE id = $1")
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
        let result = sqlx::query_as("SELECT * FROM user_progress WHERE id = $1")
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
        let result = sqlx::query_as("SELECT * FROM user_progress LIMIT $1 OFFSET $2")
            .bind(limit)
            .bind(offset)
            .fetch_all(mm.executor())
            .await?;
        Ok(result)
    }

    async fn count(mm: &ModelManager, _actor: &AuthenticatedUser) -> DatabaseResult<i64> {
        let result: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM user_progress")
            .fetch_one(mm.executor())
            .await?;

        Ok(result)
    }
}

impl_paginatable_for!(UserProgress, UserProgressCreate, Uuid);

#[async_trait]
impl HasOwner for UserProgress {
    type OwnerId = uuid::Uuid;

    async fn get_owner_id(
        &self,
        _mm: &ModelManager,
        _actor: &AuthenticatedUser,
    ) -> DatabaseResult<Self::OwnerId> {
        Ok(self.user_id)
    }
}
