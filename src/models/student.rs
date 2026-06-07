use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Student {
    pub name: String,
    pub matriculation_number: String,
    pub birthdate: String,
    pub present: bool,
    pub in_restroom: bool,
    pub restroom_entered_at: Option<chrono::DateTime<chrono::Local>>,
    pub notes: Vec<String>,
}

impl Student {
    pub fn new(name: String, matriculation_number: String, birthdate: String) -> Self {
        Self {
            name,
            matriculation_number,
            birthdate,
            present: false,
            in_restroom: false,
            restroom_entered_at: None,
            notes: Vec::new(),
        }
    }
}

/// Parse students from a CSV string (Incampo export format)
/// Expected columns: name, matriculation_number, birthdate
pub fn parse_students_from_csv(content: &str) -> Result<Vec<Student>, String> {
    let mut students = Vec::new();
    let mut lines = content.lines();

    // Skip header row
    lines.next();

    for (i, line) in lines.enumerate() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }

        let parts: Vec<&str> = line.splitn(3, ',').collect();

        if parts.len() < 3 {
            return Err(format!(
                "Line {}: expected 3 columns (name, matno, birthdate), got {}",
                i + 2,
                parts.len()
            ));
        }

        students.push(Student::new(
            parts[0].trim().to_string(),
            parts[1].trim().to_string(),
            parts[2].trim().to_string(),
        ));
    }

    Ok(students)
}