use chrono::{DateTime, Local};

use super::exam::Exam;
use super::exam_event::{EventKind, ExamEvent};
use super::student::Student;

pub struct AppState {
    pub exam: Exam,
    pub students: Vec<Student>,
    pub timer_running: bool,
    pub events: Vec<ExamEvent>,
    pub started_at: Option<DateTime<Local>>,
    /// Tracks whether ExamEnded has already been logged (avoids duplicates).
    pub exam_ended: bool,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            exam: Exam::default(),
            students: Vec::new(),
            timer_running: false,
            events: Vec::new(),
            started_at: None,
            exam_ended: false,
        }
    }
}

impl AppState {
    /// Push a timestamped event into the log.
    pub fn log_event(&mut self, kind: EventKind) {
        self.events.push(ExamEvent::new(kind));
    }

    /// Wipe everything back to a blank slate, ready for a new exam session.
    pub fn reset(&mut self) {
        *self = Self::default();
    }

    /// Indices of students matching the query (name, matriculation number, or birthdate).
    pub fn matching_student_indices(&self, query: &str) -> Vec<usize> {
        let query = query.trim();
        if query.is_empty() {
            return (0..self.students.len()).collect();
        }

        let query = query.to_lowercase();
        self.students
            .iter()
            .enumerate()
            .filter(|(_, student)| {
                student.name.to_lowercase().contains(&query)
                    || student
                        .matriculation_number
                        .to_lowercase()
                        .contains(&query)
                    || student.birthdate.to_lowercase().contains(&query)
            })
            .map(|(index, _)| index)
            .collect()
    }
}
