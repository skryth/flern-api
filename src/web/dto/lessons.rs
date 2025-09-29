use serde::Serialize;
use uuid::Uuid;

use crate::model::entity::LessonWithStatusRow;

#[derive(Debug, Clone, Serialize, utoipa::ToSchema)]
pub struct LessonResponse {
    id: Uuid,
    module_id: Uuid,
    title: String,
    content: String,
    status: bool,
    order_index: i32,
}

impl From<LessonWithStatusRow> for LessonResponse {
    fn from(row: LessonWithStatusRow) -> Self {
        Self {
            id: row.id,
            module_id: row.module_id,
            title: row.title,
            content: row.content,
            status: row.status,
            order_index: row.order_index,
        }
    }
}
