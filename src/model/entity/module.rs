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
pub struct Module {
    id: uuid::Uuid,
    title: String,
    description: String,
    order_index: i32,
}

#[derive(Debug, Deserialize, Serialize, utoipa::ToSchema)]
pub struct ModuleCreate {
    title: String,
    description: String,
    order_index: Option<i32>,
}

impl ResourceTyped for Module {
    fn get_resource_type() -> crate::model::ResourceType {
        crate::model::ResourceType::Module
    }
}

impl Module {
    pub fn new(id: Uuid, title: String, description: String, order_index: i32) -> Self {
        Self {
            id,
            title,
            description,
            order_index,
        }
    }

    pub fn id(&self) -> uuid::Uuid {
        self.id
    }

    pub fn title(&self) -> &str {
        &self.title
    }

    pub fn description(&self) -> &str {
        &self.description
    }

    pub fn order_index(&self) -> i32 {
        self.order_index
    }
}

#[async_trait]
impl CrudRepository<Module, ModuleCreate, uuid::Uuid> for Module {
    async fn create(
        mm: &ModelManager,
        _actor: &AuthenticatedUser,
        data: ModuleCreate,
    ) -> DatabaseResult<Self> {
        let result = sqlx::query("INSERT INTO modules (id, title, description, order_index) VALUES ($1,$2,$3,$4) RETURNING id")
            .bind(Uuid::new_v4())
            .bind(&data.title)
            .bind(&data.description)
            .bind(data.order_index.unwrap_or(0))
            .fetch_one(mm.executor())
            .await?;

        let id = result.try_get("id")?;
        Ok(Module {
            id,
            title: data.title,
            description: data.description,
            order_index: data.order_index.unwrap_or(0),
        })
    }

    async fn update(
        mut self,
        mm: &ModelManager,
        _actor: &AuthenticatedUser,
        data: ModuleCreate,
    ) -> DatabaseResult<Self> {
        sqlx::query(
            "UPDATE modules SET title = $1, description = $2, order_index = $3 WHERE id = $4",
        )
        .bind(&data.title)
        .bind(&data.description)
        .bind(&data.order_index.unwrap_or(0))
        .bind(self.id)
        .execute(mm.executor())
        .await?;

        self.title = data.title;
        self.description = data.description;
        self.order_index = data.order_index.unwrap_or(0);
        Ok(self)
    }

    async fn delete(self, mm: &ModelManager, _actor: &AuthenticatedUser) -> DatabaseResult<()> {
        sqlx::query("DELETE FROM modules WHERE id = $1")
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
        let result = sqlx::query_as("SELECT * FROM modules WHERE id = $1")
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
        let result = sqlx::query_as("SELECT * FROM modules LIMIT $1 OFFSET $2")
            .bind(limit)
            .bind(offset)
            .fetch_all(mm.executor())
            .await?;
        Ok(result)
    }

    async fn count(mm: &ModelManager, _actor: &AuthenticatedUser) -> DatabaseResult<i64> {
        let result: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM modules")
            .fetch_one(mm.executor())
            .await?;

        Ok(result)
    }
}

impl Module {
    pub async fn all(
        mm: &ModelManager,
        _actor: &AuthenticatedUser
    ) -> DatabaseResult<Vec<Self>> {
        let result = sqlx::query_as("SELECT * FROM modules")
            .fetch_all(mm.executor())
            .await?;
        Ok(result)
    }
}

impl_paginatable_for!(Module, ModuleCreate, Uuid);

#[async_trait]
impl HasOwner for Module {
    type OwnerId = uuid::Uuid;

    async fn get_owner_id(
        &self,
        _mm: &ModelManager,
        _actor: &AuthenticatedUser,
    ) -> DatabaseResult<Self::OwnerId> {
        Ok(self.id) // owners of modules are themselves
    }
}


// Utils

#[derive(sqlx::FromRow)]
pub struct ModuleWithLessonsRow {
    pub id: Uuid,
    pub title: String,
    pub description: String,
    pub order_index: i32,
    pub lessons: serde_json::Value,
}

impl ModuleWithLessonsRow {
    pub async fn fetch_all(
        mm: &ModelManager,
        actor: &AuthenticatedUser,
    ) -> DatabaseResult<Vec<Self>> {
        let rows: Vec<ModuleWithLessonsRow> = sqlx::query_as(
            r#"
            SELECT
            m.id,
            m.title,
            m.description,
            m.order_index,
            COALESCE(
                json_agg(
                    json_build_object(
                        'id', l.id,
                        'title', l.title,
                        'completed', COALESCE(up.status = 'done', false)
                    )
                ) FILTER (WHERE l.id IS NOT NULL),
                '[]'
            ) AS lessons
            FROM modules m
            LEFT JOIN lessons l ON l.module_id = m.id
            LEFT JOIN user_progress up
            ON up.lesson_id = l.id
            AND up.user_id = $1
            GROUP BY m.id
            ORDER BY m.order_index;
        "#
            )
            .bind(actor.user_id())
            .fetch_all(mm.executor())
            .await?;

        Ok(rows) 
    }
}

