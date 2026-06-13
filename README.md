# Exam Companion

A desktop app to manage in-person exams — track attendance, log events in real time, add notes, and export a full session report as a Markdown file.

---

## Features

- **Exam setup** — enter exam name, professor name, and duration before starting
- **Student list** — import students from a CSV file, mark who is present
- **Student details** — toggle restroom status per student, add per-student notes
- **Live timer window** — a separate fullscreen-friendly countdown shown to students
- **Real-time notes** — exam-wide notes appear instantly on the student timer window
- **Additional time** — adjust duration with +/− controls during the exam; changes are logged
- **Event log** — every action (exam start/end, restroom visits, notes, time changes) is timestamped automatically
- **Mid-exam export** — export the current report at any point without interrupting the exam
- **End Exam flow** — dedicated End Exam button triggers export then resets everything for the next session

---

## Running Instructions

### Prerequisites

You need the following installed on your system:

| Requirement | Version |
|---|---|
| Rust + Cargo | 1.85+ (edition 2024) |
| GTK4 | 4.10+ |
| libadwaita | 1.5+ |

**On Debian / Ubuntu:**
```bash
sudo apt install libgtk-4-dev libadwaita-1-dev
```

**On Fedora:**
```bash
sudo dnf install gtk4-devel libadwaita-devel
```

**On Arch:**
```bash
sudo pacman -S gtk4 libadwaita
```

**On macOS (Homebrew):**
```bash
brew install gtk4 libadwaita
```

### Build & Run

```bash
git clone <repo-url>
cd rust_summer26_05
cargo run
```

For a release build:
```bash
cargo build --release
./target/release/rust_summer26_05
```

---

## Usage

1. **Setup screen** — fill in the exam name, professor name, and adjust the duration if needed
2. **Import students** — click "Import Student List (CSV)" and select your file
3. **Mark attendance** — check "In exam" next to each student who is present
4. **Save & Start** — opens the student-facing timer window and starts the countdown
5. **During the exam:**
   - Use the student details panel to toggle restroom status and add notes per student
   - Add exam-wide notes (they appear live on the timer window)
   - Adjust time with +/− if needed
   - Click **Export Report** at any time to save a snapshot — no reset, exam keeps running
6. **End Exam** — click the red "End Exam" button, confirm, choose a save location → report is saved, state resets, app returns to a fresh setup screen

---

## CSV Format

The student import expects a plain CSV with a header row and three columns:

```
name,matriculation_number,birthdate
Alice Müller,12345678,2001-04-12
Bob Meier,87654321,2000-09-03
```

---

## Exported Report Format

The `.md` report contains four sections:

- **Exam header** — name, professor, date, start time, duration
- **Attendance table** — every student with ✅ Present / ❌ Absent and a summary count
- **Event log** — timestamped table of every action during the session
- **Student notes** — per-student notes grouped by name
- **Exam notes** — the exam-wide notes broadcast during the session

---

## Libraries

| Crate | Purpose |
|---|---|
| `gtk4` (`gtk`) | UI widgets and layout |
| `libadwaita` (`adw`) | GNOME-style components (cards, dialogs, header bars) |
| `serde` | Serialize / deserialize data structures |
| `chrono` | Wall-clock timestamps on events |

---

## Project Structure

```
src/
├── main.rs                         # App entry point, wires state and GTK application
│
├── models/
│   ├── mod.rs
│   ├── app_state.rs                # Central shared state (exam, students, events)
│   ├── exam.rs                     # Exam metadata (name, professor, duration, notes)
│   ├── exam_event.rs               # Timestamped event log (ExamEvent + EventKind)
│   └── student.rs                  # Student record + CSV parser
│
├── data/
│   ├── mod.rs
│   ├── import.rs                   # CSV file import dialog
│   └── export.rs                   # Markdown report generator
│
└── ui/
    ├── mod.rs
    ├── home.rs                     # Professor setup screen (main window)
    ├── exam_window.rs              # Student-facing countdown timer window
    ├── import.rs                   # CSV import dialog wiring
    ├── student_list.rs             # Searchable student list panel
    ├── status.rs                   # Status page helper
    └── components/
        ├── mod.rs
        ├── exam_details_card.rs    # Exam name + professor entry fields
        ├── timer_card.rs           # Duration picker with +/− controls
        ├── notes_card.rs           # Exam-wide notes input
        └── student_details.rs      # Per-student restroom toggle + notes
```
