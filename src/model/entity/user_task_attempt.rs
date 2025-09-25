
use crate::impl_paginatable_for;
use crate::model::access::HasOwner;
use crate::model::repo::ResourceTyped;
use crate::model::{ModelManager, error::DatabaseResult, repo::CrudRepository};
use crate::web::AuthenticatedUser;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use sqlx::prelude::FromRow;
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, FromRow, utoipa::ToSchema)]
pub struct UserTaskAttempt {
    id: Uuid,
    user_id: Uuid,
    task_id: Uuid,
    selected_answer_id: Uuid,
    is_correct: bool,
}

impl ResourceTyped for UserTaskAttempt {
    fn get_resource_type() -> crate::model::ResourceType {
        crate::model::ResourceType::UserTaskAttempt
    }
}

impl UserTaskAttempt {
    pub fn id(&self) -> Uuid {
        self.id
    }

    pub fn user_id(&self) -> Uuid {
        self.user_id
    }

    pub fn task_id(&self) -> Uuid {
        self.task_id
    }

    pub fn selected_answer_id(&self) -> Uuid {
        self.selected_answer_id
    }

    pub fn is_correct(&self) -> bool {
        self.is_correct
    }
}

pub struct UserTaskAttemptCreate {
    pub user_id: Uuid,
    pub task_id: Uuid,
    pub selected_answer_id: Uuid,
    pub is_correct: bool,
}

impl UserTaskAttemptCreate {
    pub fn new(user_id: Uuid, task_id: Uuid, selected_answer_id: Uuid, is_correct: bool) -> Self {
        Self {
            user_id,
            task_id,
            selected_answer_id,
            is_correct,
        }
    }
}

#[async_trait]
impl CrudRepository<UserTaskAttempt, UserTaskAttemptCreate, uuid::Uuid> for UserTaskAttempt {
    async fn create(
        mm: &ModelManager,
        _actor: &AuthenticatedUser,
        data: UserTaskAttemptCreate,
    ) -> DatabaseResult<Self> {
        let row = sqlx::query_as(
            r#"
            INSERT INTO user_task_attempts (id, user_id, task_id, selected_answer_id, is_correct)
            VALUES ($1,$2,$3,$4,$5)
            RETURNING id, user_id, task_id, selected_answer_id, is_correct 
            "#
        )
        .bind(Uuid::new_v4())
        .bind(data.user_id)
        .bind(data.task_id)
        .bind(data.selected_answer_id)
        .bind(data.is_correct)
        .fetch_one(mm.executor())
        .await?;

        Ok(row)
    }

    async fn update(
        mut self,
        mm: &ModelManager,
        _actor: &AuthenticatedUser,
        data: UserTaskAttemptCreate,
    ) -> DatabaseResult<Self> {
        sqlx::query(
            "UPDATE user_task_attempts SET user_id = $1, task_id = $2, selected_answer_id = $3, is_correct = $4 WHERE id = $5",
        )
        .bind(data.user_id)
        .bind(data.task_id)
        .bind(data.selected_answer_id)
        .bind(data.is_correct)
        .bind(self.id)
        .execute(mm.executor())
        .await?;

        self.user_id = data.user_id;
        self.task_id = data.task_id;
        self.selected_answer_id = data.selected_answer_id;
        self.is_correct = data.is_correct;
        Ok(self)
    }

    async fn delete(self, mm: &ModelManager, _actor: &AuthenticatedUser) -> DatabaseResult<()> {
        sqlx::query("DELETE FROM user_task_attempts WHERE id = $1")
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
        let result = sqlx::query_as("SELECT * FROM user_task_attempts WHERE id = $1")
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
        let result = sqlx::query_as("SELECT * FROM user_task_attempts LIMIT $1 OFFSET $2")
            .bind(limit)
            .bind(offset)
            .fetch_all(mm.executor())
            .await?;
        Ok(result)
    }

    async fn count(mm: &ModelManager, actor: &AuthenticatedUser) -> DatabaseResult<i64> {
        let result: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM user_task_attempts WHERE user_id = $1")
            .bind(actor.user_id())
            .fetch_one(mm.executor())
            .await?;

        Ok(result)
    }
}

impl UserTaskAttempt {
    pub async fn count_correct(mm: &ModelManager, actor: &AuthenticatedUser) -> DatabaseResult<i64> {
        let result: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM user_task_attempts WHERE user_id = $1 AND is_correct = TRUE")
            .bind(actor.user_id())
            .fetch_one(mm.executor())
            .await?;
        Ok(result)
    }
}

impl_paginatable_for!(UserTaskAttempt, UserTaskAttemptCreate, Uuid);

#[async_trait]
impl HasOwner for UserTaskAttempt {
    type OwnerId = uuid::Uuid;

    async fn get_owner_id(
        &self,
        _mm: &ModelManager,
        _actor: &AuthenticatedUser,
    ) -> DatabaseResult<Self::OwnerId> {
        Ok(self.user_id)
    }
}
