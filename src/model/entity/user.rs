use crate::impl_paginatable_for;
use crate::model::access::HasOwner;
use crate::model::repo::ResourceTyped;
use crate::web::AuthenticatedUser;
use crate::web::UserRole;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use sqlx::prelude::FromRow;
use sqlx::prelude::Row;
use uuid::Uuid;

use crate::model::{ModelManager, error::DatabaseResult, repo::CrudRepository};

#[derive(Debug, Serialize, Deserialize, FromRow, utoipa::ToSchema)]
pub struct UserEntity {
    id: uuid::Uuid,
    username: String,
    #[serde(skip)]
    password_hash: String,
    role: String,
}

#[derive(Debug, Deserialize, Serialize, utoipa::ToSchema)]
pub struct UserEntityCreateUpdate {
    pub username: String,
    pub password_hash: String,
}

impl ResourceTyped for UserEntity {
    fn get_resource_type() -> crate::model::repo::ResourceType {
        crate::model::repo::ResourceType::User
    }
}

impl UserEntity {
    pub fn id(&self) -> uuid::Uuid {
        self.id
    }

    pub fn username(&self) -> &str {
        &self.username
    }

    pub fn hash(&self) -> &str {
        &self.password_hash
    }

    pub fn role(&self) -> UserRole {
        UserRole::from(self.role.as_str())
    }
}

#[async_trait::async_trait]
impl CrudRepository<UserEntity, UserEntityCreateUpdate, uuid::Uuid> for UserEntity {
    async fn create(
        mm: &ModelManager,
        _actor: &AuthenticatedUser,
        data: UserEntityCreateUpdate,
    ) -> DatabaseResult<Self> {
        let role = UserRole::User.to_string();
        let result = sqlx::query("INSERT INTO users (id, username, password_hash, role) VALUES ($1,$2,$3,$4) RETURNING id")
            .bind(Uuid::new_v4())
            .bind(&data.username)
            .bind(&data.password_hash)
            .bind(&role)
            .fetch_one(mm.executor())
            .await?;

        let id = result.try_get("id")?;
        Ok(UserEntity {
            id,
            username: data.username,
            password_hash: data.password_hash,
            role,
        })
    }

    async fn update(
        mut self,
        mm: &ModelManager,
        _actor: &AuthenticatedUser,
        data: UserEntityCreateUpdate,
    ) -> DatabaseResult<Self> {
        sqlx::query("UPDATE users SET username = $1 WHERE id = $2")
            .bind(&data.username)
            .bind(&self.id)
            .execute(mm.executor())
            .await?;

        self.password_hash = data.password_hash;
        self.username = data.username;
        Ok(self)
    }

    async fn delete(self, mm: &ModelManager, _actor: &AuthenticatedUser) -> DatabaseResult<()> {
        sqlx::query("DELETE FROM users WHERE id = $1")
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
        let result = sqlx::query_as("SELECT * FROM users WHERE id = $1")
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
        let result = sqlx::query_as("SELECT * FROM users LIMIT $1 OFFSET $2")
            .bind(limit)
            .bind(offset)
            .fetch_all(mm.executor())
            .await?;
        Ok(result)
    }

    async fn count(mm: &ModelManager, _actor: &AuthenticatedUser) -> DatabaseResult<i64> {
        let result: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM users")
            .fetch_one(mm.executor())
            .await?;

        Ok(result)
    }
}

impl_paginatable_for!(UserEntity, UserEntityCreateUpdate, Uuid);

#[async_trait]
impl HasOwner for UserEntity {
    type OwnerId = uuid::Uuid;

    async fn get_owner_id(
        &self,
        _mm: &ModelManager,
        _actor: &AuthenticatedUser,
    ) -> DatabaseResult<Self::OwnerId> {
        Ok(self.id) // owners of users are themselves
    }
}

impl UserEntity {
    pub async fn find_by_username(
        mm: &ModelManager,
        _actor: &AuthenticatedUser,
        username: &str,
    ) -> DatabaseResult<Option<Self>> {
        let result = sqlx::query_as("SELECT * FROM users WHERE username = $1")
            .bind(username)
            .fetch_one(mm.executor())
            .await;
        if let Err(sqlx::Error::RowNotFound) = result {
            return Ok(None);
        }
        Ok(Some(result?))
    }
}
