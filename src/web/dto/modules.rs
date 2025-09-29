use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{
    model::{DatabaseResult, entity::ModuleWithLessonsRow},
};

#[derive(Serialize, Deserialize, utoipa::ToSchema)]
pub struct LessonShort {
    pub id: Uuid,
    pub title: String,
    pub completed: bool,
    pub order_index: i32,
}

#[derive(Serialize, Deserialize, utoipa::ToSchema)]
pub struct ModuleWithLessons {
    pub id: Uuid,
    pub title: String,
    pub description: String,
    pub order_index: i32,
    pub lessons: Vec<LessonShort>,
}

impl TryFrom<ModuleWithLessonsRow> for ModuleWithLessons {
    type Error = serde_json::Error;

    fn try_from(value: ModuleWithLessonsRow) -> Result<Self, Self::Error> {
        Ok(Self {
            id: value.id,
            title: value.title,
            description: value.description,
            order_index: value.order_index,
            lessons: serde_json::from_value(value.lessons)?,
        })
    }
}

impl ModuleWithLessons {
    pub fn from_rows(rows: Vec<ModuleWithLessonsRow>) -> DatabaseResult<Vec<Self>> {
        Ok(rows
            .into_iter()
            .map(ModuleWithLessons::try_from)
            .collect::<Result<_, _>>()?)
    }
}
