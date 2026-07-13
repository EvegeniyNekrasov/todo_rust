use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Todo {
    pub id: i64,
    pub title: String,
    pub finished: bool,
}

impl Todo {
    pub fn new(id: i64, title: String) -> Self {
        Self {
            id,
            title,
            finished: false,
        }
    }

    pub fn toggle(&mut self) {
        self.finished = !self.finished;
    }
}
