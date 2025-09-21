use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::model::entity::{Answer, LessonTask};

#[derive(Serialize, Deserialize, utoipa::ToSchema)]
pub struct TaskResponse {
    id: Uuid,
    question: String,
    task_type: String,
    answers: Vec<AnswerResponse>,
}

#[derive(Serialize, Deserialize, utoipa::ToSchema)]
pub struct AnswerResponse {
    id: Uuid,
    #[serde(skip_serializing_if = "Option::is_none")] 
    answer_text: Option<String>,
}

impl TaskResponse {
    pub fn from_entity(task: LessonTask, answers: Vec<Answer>) -> Self {
        let hide_text = task.task_type() == "string_cmp";

        Self {
            id: task.id(),
            question: task.question().to_string(),
            task_type: task.task_type().to_string(),
            answers: answers.into_iter().map(|a| AnswerResponse {
                id: a.id(),
                answer_text: if hide_text { None } else { Some(a.answer_text().to_string()) },
            }).collect()
        }
    }
}


// TaskCheck
#[derive(Serialize, Deserialize, utoipa::ToSchema)]
pub struct TaskCheckRequest {
    pub answer_id: Uuid,
    pub task_type: String,
    pub user_answer: Option<String>,
}

#[derive(Serialize, utoipa::ToSchema)]
pub struct TaskCheckResponse {
    pub is_correct: bool,
    pub explanation: String,
    pub image: String,
}
