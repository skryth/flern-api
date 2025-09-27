use serde::Serialize;

#[derive(Serialize, utoipa::ToSchema)]
pub struct UserProgressResponse {
    total_lessons: i64,
    completed_lessons: i64,
    correct_answers: i64,
    total_answers: i64,
    username: String,
}

impl UserProgressResponse {
    pub fn new(
        total_lessons: i64,
        completed_lessons: i64,
        correct_answers: i64,
        total_answers: i64,
        username: String,
    ) -> Self {
        Self {
            total_lessons,
            completed_lessons,
            correct_answers,
            total_answers,
            username,
        }
    }
}
