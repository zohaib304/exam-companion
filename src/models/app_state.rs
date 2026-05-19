use super::student::Student;
use super::exam::Exam;

pub struct AppState {
    pub exam: Exam,
    pub students: Vec<Student>,
    pub timer_running: bool,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            exam: Exam::default(),
            students: Vec::new(),
            timer_running: false,
        }
    }
}

impl AppState {
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