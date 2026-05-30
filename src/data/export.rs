use crate::models::app_state::AppState;
use crate::models::exam_event::EventKind;

/// Build a full Markdown exam report from the current app state.
pub fn export_to_markdown(state: &AppState) -> String {
    let mut md = String::new();

    // ── Header ────────────────────────────────────────────────
    let exam_name = if state.exam.name.trim().is_empty() {
        "Untitled Exam"
    } else {
        state.exam.name.trim()
    };

    let date_str = state
        .started_at
        .map(|dt| dt.format("%Y-%m-%d").to_string())
        .unwrap_or_else(|| "Unknown".to_string());

    let start_time_str = state
        .started_at
        .map(|dt| dt.format("%H:%M:%S").to_string())
        .unwrap_or_else(|| "Unknown".to_string());

    let duration_mins = state.exam.duration_secs / 60;

    md.push_str(&format!("# Exam Report: {}\n\n", exam_name));
    md.push_str(&format!(
        "**Professor:** {}  \n",
        if state.exam.professor.trim().is_empty() {
            "—"
        } else {
            state.exam.professor.trim()
        }
    ));
    md.push_str(&format!("**Date:** {}  \n", date_str));
    md.push_str(&format!("**Started at:** {}  \n", start_time_str));
    md.push_str(&format!("**Duration:** {} min  \n", duration_mins));
    md.push_str("\n---\n\n");

    // ── Attendance ────────────────────────────────────────────
    md.push_str("## Attendance\n\n");

    if state.students.is_empty() {
        md.push_str("*No students were imported.*\n\n");
    } else {
        md.push_str("| Student | Matriculation No. | Birthdate | Status |\n");
        md.push_str("|---------|-------------------|-----------|--------|\n");
        for s in &state.students {
            let status = if s.present { "✅ Present" } else { "❌ Absent" };
            md.push_str(&format!(
                "| {} | {} | {} | {} |\n",
                s.name, s.matriculation_number, s.birthdate, status
            ));
        }
        md.push('\n');

        let present = state.students.iter().filter(|s| s.present).count();
        let total = state.students.len();
        let absent = total - present;
        md.push_str(&format!(
            "**Summary:** {present} present · {absent} absent · {total} registered\n\n"
        ));
    }

    md.push_str("---\n\n");

    // ── Event Log ─────────────────────────────────────────────
    md.push_str("## Event Log\n\n");

    if state.events.is_empty() {
        md.push_str("*No events were recorded.*\n\n");
    } else {
        md.push_str("| Time | Event |\n");
        md.push_str("|------|-------|\n");
        for event in &state.events {
            let time = event.timestamp.format("%H:%M:%S").to_string();
            let description = describe_event(&event.kind);
            md.push_str(&format!("| {} | {} |\n", time, description));
        }
        md.push('\n');
    }

    md.push_str("---\n\n");

    // ── Student Notes ─────────────────────────────────────────
    md.push_str("## Student Notes\n\n");

    let students_with_notes: Vec<_> = state.students.iter().filter(|s| !s.notes.is_empty()).collect();

    if state.students.is_empty() {
        md.push_str("*No students were imported.*\n\n");
    } else if students_with_notes.is_empty() {
        md.push_str("*No student notes were recorded.*\n\n");
    } else {
        for s in students_with_notes {
            md.push_str(&format!(
                "### {} ({})\n\n",
                s.name, s.matriculation_number
            ));
            for note in &s.notes {
                md.push_str(&format!("- {}\n", note));
            }
            md.push('\n');
        }
    }

    md.push_str("---\n\n");

    // ── Exam Notes ────────────────────────────────────────────
    md.push_str("## Exam Notes\n\n");

    if state.exam.notes.is_empty() {
        md.push_str("*No exam-wide notes were recorded.*\n\n");
    } else {
        for note in &state.exam.notes {
            md.push_str(&format!("- {}\n", note));
        }
        md.push('\n');
    }

    md
}

fn describe_event(kind: &EventKind) -> String {
    match kind {
        EventKind::ExamStarted => "Exam started".to_string(),
        EventKind::ExamEnded => "Exam ended".to_string(),
        EventKind::TimeExtended {
            added_secs,
            new_total_secs,
        } => {
            let added_mins = added_secs / 60;
            let new_total_mins = new_total_secs / 60;
            format!(
                "+{added_mins} min additional time granted (new total: {new_total_mins} min)"
            )
        }
        EventKind::StudentEnteredRestroom {
            name,
            matriculation_number,
        } => format!("{name} ({matriculation_number}) entered restroom"),
        EventKind::StudentLeftRestroom {
            name,
            matriculation_number,
        } => format!("{name} ({matriculation_number}) left restroom"),
        EventKind::StudentNoteAdded {
            name,
            matriculation_number,
            note,
        } => format!("Note added for {name} ({matriculation_number}): \"{note}\""),
        EventKind::ExamNoteAdded { note } => {
            format!("Exam note added: \"{note}\"")
        }
    }
}
