use serde::{Deserialize, Serialize};

use crate::{
    model::{ModelManager, error::DatabaseResult},
    web::AuthenticatedUser,
};

#[derive(Debug, Clone)]
pub enum ResourceType {
    User,
    Module,
    Lesson,
    Task,
    Answer,
    UserProgress,
}

#[derive(Debug, Clone, Serialize, Deserialize, utoipa::ToSchema)]
pub struct Page<T> {
    pub items: Vec<T>,
    pub total: i64,
    pub limit: i64,
    pub offset: i64,
}

impl<T> Page<T> {
    pub fn new(items: Vec<T>, total: i64, limit: i64, offset: i64) -> Self {
        Self {
            items,
            total,
            limit,
            offset,
        }
    }
}

pub trait ResourceTyped {
    fn get_resource_type() -> ResourceType;
}

#[async_trait::async_trait]
pub trait CrudRepository<T, CreateUpdate, V>
where
    T: ResourceTyped,
    V: Clone + Copy,
{
    async fn create(
        mm: &ModelManager,
        actor: &AuthenticatedUser,
        data: CreateUpdate,
    ) -> DatabaseResult<T>;
    async fn update(
        self,
        mm: &ModelManager,
        actor: &AuthenticatedUser,
        data: CreateUpdate,
    ) -> DatabaseResult<T>
    where
        Self: Sized;

    async fn delete(self, mm: &ModelManager, actor: &AuthenticatedUser) -> DatabaseResult<()>
    where
        Self: Sized;

    async fn find_by_id(
        mm: &ModelManager,
        actor: &AuthenticatedUser,
        id: V,
    ) -> DatabaseResult<Option<T>>;
    async fn list(
        mm: &ModelManager,
        actor: &AuthenticatedUser,
        limit: i64,
        offset: i64,
    ) -> DatabaseResult<Vec<T>>;
    async fn count(mm: &ModelManager, actor: &AuthenticatedUser) -> DatabaseResult<i64>;
}

#[async_trait::async_trait]
pub trait PaginatableRepository<T, CreateUpdate, V>
where
    T: ResourceTyped + CrudRepository<T, CreateUpdate, V>,
    V: Clone + Copy,
{
    async fn page(
        mm: &ModelManager,
        actor: &AuthenticatedUser,
        limit: i64,
        offset: i64,
    ) -> DatabaseResult<Page<T>>;
}

#[macro_export]
macro_rules! impl_paginatable_for {
    ($ent:ident, $ent_create:ident, $ent_id:ident) => {
        #[async_trait::async_trait]
        impl $crate::model::PaginatableRepository<$ent, $ent_create, $ent_id> for $ent {
            async fn page(
                mm: &ModelManager,
                actor: &AuthenticatedUser,
                limit: i64,
                offset: i64,
            ) -> DatabaseResult<$crate::model::Page<$ent>> {
                let items = $ent::list(mm, actor, limit, offset).await?;
                let count = $ent::count(mm, actor).await?;
                Ok($crate::model::Page::new(items, count, limit, offset))
            }
        }
    };
}
