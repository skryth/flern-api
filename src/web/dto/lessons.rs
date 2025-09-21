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
}

impl From<LessonWithStatusRow> for LessonResponse {
    fn from(row: LessonWithStatusRow) -> Self {
        Self {
            id: row.id,
            module_id: row.module_id,
            title: row.title,
            content: row.content,
            status: row.status,
        }
    }
}
