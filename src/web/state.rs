use crate::model::ModelManager;

#[derive(Debug, Clone)]
pub struct AppState {
    mm: ModelManager,
}

impl AppState {
    pub fn new(mm: ModelManager) -> Self {
        Self { mm }
    }

    pub fn pool(&self) -> &ModelManager {
        &self.mm
    }
}
