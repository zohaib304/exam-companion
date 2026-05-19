use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Exam {
    pub name: String,
    pub professor: String,
    pub duration_secs: u32,
}

impl Default for Exam {
    fn default() -> Self {
        Self {
            name: String::from("Untitled Exam"),
            professor: String::from(""),
            duration_secs: 90 * 60, // 90 minutes
        }
    }
}