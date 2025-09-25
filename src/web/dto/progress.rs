use serde::Serialize;

#[derive(Serialize, utoipa::ToSchema)]
pub struct UserProgressResponse {
    total_lessons: i64,
    completed_lessons: i64,
    correct_answers: i64,
    total_answers: i64,
}

impl UserProgressResponse {
    pub fn new(
        total_lessons: i64,
        completed_lessons: i64,
        correct_answers: i64,
        total_answers: i64,
    ) -> Self {
        Self {
            total_lessons,
            completed_lessons,
            correct_answers,
            total_answers,
        }
    }
}
