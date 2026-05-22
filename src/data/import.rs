use std::fs;
use std::path::Path;

use crate::models::student::{parse_students_from_csv, Student};

/// Read a CSV directly from the selected path and return parsed students.
pub fn import_student_csv(source: &Path) -> Result<Vec<Student>, String> {
    if !source.is_file() {
        return Err("Selected path is not a file.".into());
    }

    let content = fs::read_to_string(source)
        .map_err(|e| format!("Could not read CSV: {e}"))?;

    let students = parse_students_from_csv(&content)?;

    if students.is_empty() {
        return Err("CSV contains no student rows.".into());
    }

    Ok(students)
}