use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use crate::models::student::{parse_students_from_csv, Student};

pub const UPLOAD_DIR: &str = "upload";

/// Ensure the upload directory exists (created next to the working directory).
pub fn ensure_upload_dir() -> Result<PathBuf, String> {
    let dir = PathBuf::from(UPLOAD_DIR);
    fs::create_dir_all(&dir).map_err(|e| format!("Could not create upload folder: {e}"))?;
    Ok(dir)
}

/// Copy a selected CSV into `upload/`, parse it, and return students plus the stored path.
pub fn import_student_csv(source: &Path) -> Result<(Vec<Student>, PathBuf), String> {
    if !source.is_file() {
        return Err("Selected path is not a file.".into());
    }

    let upload_dir = ensure_upload_dir()?;
    let stored_path = store_copy(source, &upload_dir)?;

    let content = fs::read_to_string(&stored_path)
        .map_err(|e| format!("Could not read CSV: {e}"))?;

    let students = parse_students_from_csv(&content)?;
    if students.is_empty() {
        return Err("CSV contains no student rows.".into());
    }

    Ok((students, stored_path))
}

fn store_copy(source: &Path, upload_dir: &Path) -> Result<PathBuf, String> {
    let file_name = source
        .file_name()
        .and_then(|n| n.to_str())
        .ok_or("Invalid file name.")?;

    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|e| e.to_string())?
        .as_secs();

    let dest = upload_dir.join(format!("{timestamp}_{file_name}"));
    fs::copy(source, &dest).map_err(|e| format!("Could not save CSV to upload folder: {e}"))?;

    Ok(dest)
}
