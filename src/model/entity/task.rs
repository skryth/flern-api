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
pub struct LessonTask {
    id: Uuid,
    lesson_id: Uuid,
    task_type: String,
    question: String,
    explanation: String,
}

impl ResourceTyped for LessonTask {
    fn get_resource_type() -> crate::model::ResourceType {
        crate::model::ResourceType::Task
    }
}

impl LessonTask {
    pub fn id(&self) -> uuid::Uuid {
        self.id
    }

    pub fn lesson_id(&self) -> uuid::Uuid {
        self.lesson_id
    }

    pub fn task_type(&self) -> &str {
        &self.task_type
    }

    pub fn question(&self) -> &str {
        &self.question
    }

    pub fn explanation(&self) -> &str {
        &self.explanation
    }
}

#[derive(Debug, Deserialize, Serialize, utoipa::ToSchema)]
pub struct LessonTaskCreate {
    lesson_id: Uuid,
    task_type: String,
    question: String,
    explanation: String,
}

#[async_trait]
impl CrudRepository<LessonTask, LessonTaskCreate, uuid::Uuid> for LessonTask {
    async fn create(
        mm: &ModelManager,
        _actor: &AuthenticatedUser,
        data: LessonTaskCreate,
    ) -> DatabaseResult<Self> {
        let result = sqlx::query("INSERT INTO tasks (id, lesson_id, task_type, question, explanation) VALUES ($1,$2,$3,$4,$5) RETURNING id")
            .bind(Uuid::new_v4())
            .bind(data.lesson_id)
            .bind(&data.task_type)
            .bind(&data.question)
            .bind(&data.explanation)
            .fetch_one(mm.executor())
            .await?;

        let id = result.try_get("id")?;
        Ok(LessonTask {
            id,
            lesson_id: data.lesson_id,
            task_type: data.task_type,
            question: data.question,
            explanation: data.explanation,
        })
    }

    async fn update(
        mut self,
        mm: &ModelManager,
        _actor: &AuthenticatedUser,
        data: LessonTaskCreate,
    ) -> DatabaseResult<Self> {
        sqlx::query("UPDATE tasks SET lesson_id = $1, task_type = $2, question = $3, explanation = $4 WHERE id = $5")
            .bind(data.lesson_id)
            .bind(&data.task_type)
            .bind(&data.question)
            .bind(&data.explanation)
            .bind(self.id)
            .execute(mm.executor())
            .await?;

        self.lesson_id = data.lesson_id;
        self.task_type = data.task_type;
        self.question = data.question;
        self.explanation = data.explanation;
        Ok(self)
    }

    async fn delete(self, mm: &ModelManager, _actor: &AuthenticatedUser) -> DatabaseResult<()> {
        sqlx::query("DELETE FROM tasks WHERE id = $1")
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
        let result = sqlx::query_as("SELECT * FROM tasks WHERE id = $1")
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
        let result = sqlx::query_as("SELECT * FROM tasks LIMIT $1 OFFSET $2")
            .bind(limit)
            .bind(offset)
            .fetch_all(mm.executor())
            .await?;
        Ok(result)
    }

    async fn count(mm: &ModelManager, _actor: &AuthenticatedUser) -> DatabaseResult<i64> {
        let result: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM tasks")
            .fetch_one(mm.executor())
            .await?;

        Ok(result)
    }
}

#[async_trait]
impl HasOwner for LessonTask {
    type OwnerId = uuid::Uuid;

    async fn get_owner_id(
        &self,
        _mm: &ModelManager,
        _actor: &AuthenticatedUser,
    ) -> DatabaseResult<Self::OwnerId> {
        Ok(self.lesson_id)
    }
}

// Utils
impl LessonTask {
    pub async fn find_all_by_lesson(
        mm: &ModelManager,
        _actor: &AuthenticatedUser,
        lesson_id: Uuid,
    ) -> DatabaseResult<Vec<Self>> {
        let rows: Vec<Self> = sqlx::query_as(
            r#"
            SELECT *
            FROM tasks t
            WHERE t.lesson_id = $1
            "#
        )
        .bind(lesson_id)
        .fetch_all(mm.executor())
        .await?;

        Ok(rows)
    }
}
