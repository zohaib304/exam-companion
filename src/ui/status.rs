use crate::models::app_state::AppState;

pub fn student_status_message(state: &AppState) -> (String, String) {
    if state.students.is_empty() {
        (
            "No Students Imported".to_string(),
            "Import a CSV file exported from Incampo to get started.".to_string(),
        )
    } else {
        (
            "Students Loaded".to_string(),
            format!(
                "{} students ready for the exam.",
                state.students.len()
            ),
        )
    }
}
