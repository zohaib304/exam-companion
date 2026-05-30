use chrono::{DateTime, Local};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExamEvent {
    pub timestamp: DateTime<Local>,
    pub kind: EventKind,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EventKind {
    ExamStarted,
    ExamEnded,
    TimeExtended {
        added_secs: i64,
        new_total_secs: u32,
    },
    StudentEnteredRestroom {
        name: String,
        matriculation_number: String,
    },
    StudentLeftRestroom {
        name: String,
        matriculation_number: String,
    },
    StudentNoteAdded {
        name: String,
        matriculation_number: String,
        note: String,
    },
    ExamNoteAdded {
        note: String,
    },
}

impl ExamEvent {
    pub fn new(kind: EventKind) -> Self {
        Self {
            timestamp: Local::now(),
            kind,
        }
    }
}
