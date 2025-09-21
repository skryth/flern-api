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
pub struct Lesson {
    id: Uuid,
    module_id: Uuid,
    title: String,
    content: String,
    order_index: i32,
}

impl ResourceTyped for Lesson {
    fn get_resource_type() -> crate::model::ResourceType {
        crate::model::ResourceType::Lesson
    }
}

impl Lesson {
    pub fn id(&self) -> uuid::Uuid {
        self.id
    }

    pub fn module_id(&self) -> uuid::Uuid {
        self.module_id
    }

    pub fn title(&self) -> &str {
        &self.title
    }

    pub fn content(&self) -> &str {
        &self.content
    }

    pub fn order_index(&self) -> i32 {
        self.order_index
    }
}

#[derive(Debug, Deserialize, Serialize, utoipa::ToSchema)]
pub struct LessonCreate {
    module_id: Uuid,
    title: String,
    content: String,
    order_index: Option<i32>,
}

#[async_trait]
impl CrudRepository<Lesson, LessonCreate, uuid::Uuid> for Lesson {
    async fn create(
        mm: &ModelManager,
        _actor: &AuthenticatedUser,
        data: LessonCreate,
    ) -> DatabaseResult<Self> {
        let result = sqlx::query("INSERT INTO lessons (id, module_id, title, content, order_index) VALUES ($1,$2,$3,$4,$5) RETURNING id")
            .bind(Uuid::new_v4())
            .bind(data.module_id)
            .bind(&data.title)
            .bind(&data.content)
            .bind(data.order_index.unwrap_or(0))
            .fetch_one(mm.executor())
            .await?;

        let id = result.try_get("id")?;
        Ok(Lesson {
            id,
            module_id: data.module_id,
            title: data.title,
            content: data.content,
            order_index: data.order_index.unwrap_or(0),
        })
    }

    async fn update(
        mut self,
        mm: &ModelManager,
        _actor: &AuthenticatedUser,
        data: LessonCreate,
    ) -> DatabaseResult<Self> {
        sqlx::query("UPDATE lessons SET module_id = $1, title = $2, content = $3, order_index = $4 WHERE id = $5")
            .bind(data.module_id)
            .bind(&data.title)
            .bind(&data.content)
            .bind(&data.order_index.unwrap_or(0))
            .bind(self.id)
            .execute(mm.executor())
            .await?;

        self.module_id = data.module_id;
        self.title = data.title;
        self.content = data.content;
        self.order_index = data.order_index.unwrap_or(0);
        Ok(self)
    }

    async fn delete(self, mm: &ModelManager, _actor: &AuthenticatedUser) -> DatabaseResult<()> {
        sqlx::query("DELETE FROM lessons WHERE id = $1")
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
        let result = sqlx::query_as("SELECT * FROM lessons WHERE id = $1")
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
        let result = sqlx::query_as("SELECT * FROM lessons LIMIT $1 OFFSET $2")
            .bind(limit)
            .bind(offset)
            .fetch_all(mm.executor())
            .await?;
        Ok(result)
    }

    async fn count(mm: &ModelManager, _actor: &AuthenticatedUser) -> DatabaseResult<i64> {
        let result: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM lessons")
            .fetch_one(mm.executor())
            .await?;

        Ok(result)
    }
}

impl Lesson {
    pub async fn all_by_module(
        mm: &ModelManager,
        _actor: &AuthenticatedUser,
        mid: Uuid
    ) -> DatabaseResult<Vec<Self>> {
        let result = sqlx::query_as("SELECT * FROM lessons WHERE module_id = $1")
            .bind(mid)
            .fetch_all(mm.executor())
            .await?;
        Ok(result)
    }
}

impl_paginatable_for!(Lesson, LessonCreate, Uuid);

#[async_trait]
impl HasOwner for Lesson {
    type OwnerId = uuid::Uuid;

    async fn get_owner_id(
        &self,
        _mm: &ModelManager,
        _actor: &AuthenticatedUser,
    ) -> DatabaseResult<Self::OwnerId> {
        Ok(self.module_id)
    }
}

// Utils

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct LessonWithStatusRow {
    pub id: Uuid,
    pub module_id: Uuid,
    pub title: String,
    pub content: String,
    pub status: bool,
}

impl LessonWithStatusRow {
    pub async fn find_by_id(
        mm: &ModelManager,
        actor: &AuthenticatedUser,
        lesson_id: Uuid,
    ) -> DatabaseResult<Self> {
        let row: LessonWithStatusRow = sqlx::query_as(
            r#"
            SELECT 
                l.id, 
                l.module_id, 
                l.title, 
                l.content, 
                COALESCE(up.status, false) AS status
            FROM lessons l
            LEFT JOIN user_progress up
                ON l.id = up.lesson_id AND up.user_id = $2
            WHERE l.id = $1
            "#
        )
        .bind(lesson_id)
        .bind(&actor.user_id())
        .fetch_one(mm.executor())
        .await?;

        Ok(row)
    }

    pub async fn find_next_uncompleted(
        mm: &ModelManager,
        actor: &AuthenticatedUser,
        lesson_id: Uuid,
    ) -> DatabaseResult<Option<Self>> {
        let row = sqlx::query_as(
            r#"
            SELECT
                l.id,
                l.module_id,
                l.title,
                l.content,
                COALESCE(up.status, FALSE) AS status
            FROM lessons l
            JOIN modules m ON m.id = l.module_id
            LEFT JOIN user_progress up
                ON up.lesson_id = l.id
                AND up.user_id = $2
            WHERE m.id = (
                SELECT module_id FROM lessons WHERE id = $1
            )
            AND l.order_index > (
                SELECT order_index FROM lessons WHERE id = $1
            )
            AND COALESCE (up.status, FALSE) = FALSE
            ORDER BY l.order_index ASC
            LIMIT 1;
            "#
        )
        .bind(lesson_id)
        .bind(actor.user_id())
        .fetch_optional(mm.executor())
        .await?;

        Ok(row)
    }
}
