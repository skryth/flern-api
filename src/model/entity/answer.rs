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
pub struct Answer {
    id: Uuid,
    task_id: Uuid,
    answer_text: String,
    image: String,
    is_correct: bool,
}

impl ResourceTyped for Answer {
    fn get_resource_type() -> crate::model::ResourceType {
        crate::model::ResourceType::Answer
    }
}

impl Answer {
    pub fn id(&self) -> uuid::Uuid {
        self.id
    }

    pub fn task_id(&self) -> uuid::Uuid {
        self.task_id
    }

    pub fn answer_text(&self) -> &str {
        &self.answer_text
    }

    pub fn image(&self) -> &str {
        &self.image
    }

    pub fn is_correct(&self) -> bool {
        self.is_correct
    }
}

#[derive(Debug, Deserialize, Serialize, utoipa::ToSchema)]
pub struct AnswerCreate {
    pub task_id: Uuid,
    pub answer_text: String,
    pub image: String,
    pub is_correct: Option<bool>,
}

#[async_trait]
impl CrudRepository<Answer, AnswerCreate, uuid::Uuid> for Answer {
    async fn create(
        mm: &ModelManager,
        _actor: &AuthenticatedUser,
        data: AnswerCreate,
    ) -> DatabaseResult<Self> {
        let result = sqlx::query("INSERT INTO task_answers (id, task_id, answer_text, image, is_correct) VALUES ($1,$2,$3,$4,$5) RETURNING id")
            .bind(Uuid::new_v4())
            .bind(data.task_id)
            .bind(&data.answer_text)
            .bind(&data.image)
            .bind(data.is_correct.unwrap_or(false))
            .fetch_one(mm.executor())
            .await?;

        let id = result.try_get("id")?;
        Ok(Answer {
            id,
            task_id: data.task_id,
            answer_text: data.answer_text,
            image: data.image,
            is_correct: data.is_correct.unwrap_or(false),
        })
    }

    async fn update(
        mut self,
        mm: &ModelManager,
        _actor: &AuthenticatedUser,
        data: AnswerCreate,
    ) -> DatabaseResult<Self> {
        sqlx::query("UPDATE task_answers SET task_id = $1, answer_text = $2, image = $3, is_correct = $4 WHERE id = $5")
            .bind(data.task_id)
            .bind(&data.answer_text)
            .bind(&data.image)
            .bind(data.is_correct.unwrap_or(false))
            .bind(self.id)
            .execute(mm.executor())
            .await?;

        self.task_id = data.task_id;
        self.answer_text = data.answer_text;
        self.image = data.image;
        self.is_correct = data.is_correct.unwrap_or(false);
        Ok(self)
    }

    async fn delete(self, mm: &ModelManager, _actor: &AuthenticatedUser) -> DatabaseResult<()> {
        sqlx::query("DELETE FROM task_answers WHERE id = $1")
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
        let result = sqlx::query_as("SELECT * FROM task_answers WHERE id = $1")
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
        let result = sqlx::query_as("SELECT * FROM task_answers LIMIT $1 OFFSET $2")
            .bind(limit)
            .bind(offset)
            .fetch_all(mm.executor())
            .await?;
        Ok(result)
    }

    async fn count(mm: &ModelManager, _actor: &AuthenticatedUser) -> DatabaseResult<i64> {
        let result: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM task_answers")
            .fetch_one(mm.executor())
            .await?;

        Ok(result)
    }
}

impl_paginatable_for!(Answer, AnswerCreate, Uuid);

#[async_trait]
impl HasOwner for Answer {
    type OwnerId = uuid::Uuid;

    async fn get_owner_id(
        &self,
        _mm: &ModelManager,
        _actor: &AuthenticatedUser,
    ) -> DatabaseResult<Self::OwnerId> {
        Ok(self.task_id)
    }
}

// Utils

impl Answer {
    pub async fn find_all_by_task(
        mm: &ModelManager,
        _actor: &AuthenticatedUser,
        task_id: Uuid,
    ) -> DatabaseResult<Vec<Self>> {
        let rows: Vec<Self> = sqlx::query_as(
            r#"
            SELECT *
            FROM task_answers ta
            WHERE ta.task_id = $1
            "#
        )
        .bind(task_id)
        .fetch_all(mm.executor())
        .await?;

        Ok(rows)
    }
}
