use gtk::glib;
use std::cell::RefCell;
use std::rc::Rc;

use adw::prelude::*;
use adw::{
    Application, ApplicationWindow, HeaderBar, MessageDialog, ResponseAppearance, StatusPage,
};
use gtk::gio;
use gtk::{Box, Button, FileDialog, Orientation, Separator, Stack};

use crate::data::export::export_to_markdown;
use crate::models::app_state::AppState;
use crate::models::exam_event::EventKind;
use crate::ui::components::{exam_details_card, notes_card, student_details, timer_card};
use crate::ui::{import, student_list, student_status_message};

fn get_backup_path() -> std::path::PathBuf {
    let dir = dirs::desktop_dir() // ~/Desktop
        .or_else(|| dirs::home_dir()) // ~/ as fallback
        .unwrap_or_else(|| std::path::PathBuf::from(".")) // cwd as last resort
        .join("exam-companion-backup");

    std::fs::create_dir_all(&dir).ok();

    dir.join("exam_backup.md")
}

pub fn build(app: &Application, state: Rc<RefCell<AppState>>) {
    // ─── HEADER ───────────────────────────────────────────────
    let header = HeaderBar::new();
    header.set_decoration_layout(Some("icon:minimize,maximize,close"));

    // Left: mid-exam export (always available, no reset)
    let export_btn = Button::builder()
        .label("Export Report")
        .css_classes(["pill"])
        .build();
    header.pack_start(&export_btn);

    // Right: Save & Start  ←→  End Exam  (swapped when exam is running)
    let start_btn = Button::builder()
        .label("Save & Start")
        .css_classes(["suggested-action", "pill"])
        .build();

    let end_btn = Button::builder()
        .label("End Exam")
        .css_classes(["destructive-action", "pill"])
        .visible(false)
        .build();

    header.pack_end(&start_btn);
    header.pack_end(&end_btn);

    // ─── LEFT COLUMN ──────────────────────────────────────────
    let screen_title = gtk::Label::builder()
        .label("Setup")
        .halign(gtk::Align::Start)
        .css_classes(["title-2"])
        .margin_bottom(4)
        .build();

    let left_card = exam_details_card::build(state.clone());
    let t_card = timer_card::build(state.clone());
    let n_card = notes_card::build(state.clone());
    let duration_mins = timer_card::duration_mins(state.clone());

    let left_column = Box::builder()
        .orientation(Orientation::Vertical)
        .spacing(12)
        .margin_top(24)
        .margin_start(24)
        .margin_end(12)
        .margin_bottom(24)
        .width_request(180)
        .hexpand(true)
        .vexpand(true)
        .build();
    left_column.append(&screen_title);
    left_column.append(&left_card);
    left_column.append(&t_card);
    left_column.append(&n_card);

    // ─── MIDDLE COLUMN — Student List ─────────────────────────
    let import_btn = Button::builder()
        .label("Import Student List (CSV)")
        .css_classes(["suggested-action", "pill"])
        .halign(gtk::Align::Center)
        .build();

    let (title, description) = student_status_message(&state.borrow());
    let status_page = StatusPage::builder()
        .title(&title)
        .description(&description)
        .icon_name("document-open-symbolic")
        .child(&import_btn)
        .build();

    // ─── RIGHT COLUMN — Student Details ───────────────────────
    let student_panel_handle: Rc<RefCell<Option<Rc<student_list::StudentListPanel>>>> =
        Rc::new(RefCell::new(None));

    let student_panel_handle_for_restroom = student_panel_handle.clone();
    let (details_card, on_student_selected) = student_details::build(
        state.clone(),
        Rc::new(move |index| {
            if let Some(student_panel) = student_panel_handle_for_restroom.borrow().as_ref() {
                student_panel.update_restroom_indicator(index);
            }
        }),
    );

    let student_panel = student_list::StudentListPanel::new(
        state.clone(),
        Rc::new(move |index| on_student_selected(index)),
    );
    *student_panel_handle.borrow_mut() = Some(student_panel.clone());

    let stack = Stack::builder().vexpand(true).hexpand(true).build();
    stack.add_named(&status_page, Some("empty"));
    stack.add_named(student_panel.widget(), Some("students"));
    update_stack_page(&stack, &state.borrow());

    let middle_column = Box::builder()
        .orientation(Orientation::Vertical)
        .spacing(12)
        .margin_top(24)
        .margin_start(12)
        .margin_end(12)
        .margin_bottom(24)
        .width_request(360)
        .hexpand(true)
        .vexpand(true)
        .build();
    middle_column.append(&stack);

    let right_column = Box::builder()
        .orientation(Orientation::Vertical)
        .spacing(12)
        .margin_top(24)
        .margin_start(12)
        .margin_end(24)
        .margin_bottom(24)
        .width_request(360)
        .hexpand(true)
        .vexpand(true)
        .build();
    right_column.append(&details_card);

    // ─── LAYOUT ───────────────────────────────────────────────
    let sep1 = Separator::new(Orientation::Vertical);
    let sep2 = Separator::new(Orientation::Vertical);

    let columns = Box::builder()
        .orientation(Orientation::Horizontal)
        .hexpand(true)
        .vexpand(true)
        .build();
    columns.append(&left_column);
    columns.append(&sep1);
    columns.append(&middle_column);
    columns.append(&sep2);
    columns.append(&right_column);

    let content = Box::builder().orientation(Orientation::Vertical).build();
    content.append(&header);
    content.append(&columns);

    // ─── WINDOW ───────────────────────────────────────────────
    let window = ApplicationWindow::builder()
        .application(app)
        .title("Exam Companion")
        .default_width(1100)
        .default_height(700)
        .resizable(true)
        .content(&content)
        .build();

    window.set_maximized(false);

    // Shared handle to the student-facing exam window so End Exam can close it
    let exam_window_handle: Rc<RefCell<Option<ApplicationWindow>>> = Rc::new(RefCell::new(None));

    // ─── IMPORT ───────────────────────────────────────────────
    let on_imported = {
        let stack = stack.clone();
        let student_panel = student_panel.clone();
        let state = state.clone();
        Rc::new(move || {
            update_stack_page(&stack, &state.borrow());
            student_panel.refresh();
        })
    };

    let window_for_import = window.clone();
    let state_for_import = state.clone();
    let on_imported_btn = on_imported.clone();
    import_btn.connect_clicked(move |_| {
        import::open_csv_import_dialog(
            &window_for_import,
            state_for_import.clone(),
            on_imported_btn.clone(),
        );
    });

    // ─── EXPORT (mid-exam, no reset) ──────────────────────────
    let state_for_export = state.clone();
    let window_for_export = window.clone();
    
    export_btn.connect_clicked(move |_| {
        let md = export_to_markdown(&state_for_export.borrow());
        let backup_path = get_backup_path();
        open_save_dialog(&window_for_export, md, None, Some(backup_path));
    });

    // ─── SAVE & START ─────────────────────────────────────────
    {
        let state_for_start = state.clone();
        let window_for_start = window.clone();
        let app_for_start = app.clone();
        let start_btn_clone = start_btn.clone();
        let end_btn_clone = end_btn.clone();
        let exam_window_for_start = exam_window_handle.clone();
        let duration_mins = duration_mins.clone();

        start_btn.connect_clicked(move |_| {
            let s = state_for_start.borrow();
            let error_msg = validate_start(&s);
            drop(s);

            if let Some(msg) = error_msg {
                show_error_dialog(&window_for_start, msg);
                return;
            }

            let mins = *duration_mins.borrow();
            {
                let mut s = state_for_start.borrow_mut();
                s.exam.duration_secs = mins * 60;
                s.timer_running = true;
                s.started_at = Some(chrono::Local::now());
                s.exam_ended = false;
                s.log_event(EventKind::ExamStarted);
            }

            // Swap buttons
            start_btn_clone.set_visible(false);
            end_btn_clone.set_visible(true);

            // ── AUTO-BACKUP every 5 minutes ──────────────────────────
            {
                let state_for_backup = state_for_start.clone();
                let backup_path = get_backup_path();

                glib::timeout_add_seconds_local(60, move || {
                    let s = state_for_backup.borrow();

                    if !s.timer_running || s.exam_ended {
                        return glib::ControlFlow::Break;
                    }

                    let md = export_to_markdown(&s);
                    drop(s);

                    match std::fs::write(&backup_path, md.as_bytes()) {
                        Ok(_) => eprintln!("[auto-backup] Saved → {:?}", backup_path),
                        Err(e) => eprintln!("[auto-backup] Write failed: {e}"),
                    }

                    glib::ControlFlow::Continue
                });
            }
            // ─────────────────────────────────────────────────────────

            // Open student-facing exam window and keep a handle to it
            let ew = crate::ui::exam_window::open(&app_for_start, state_for_start.clone());
            *exam_window_for_start.borrow_mut() = Some(ew);
        });
    }

    // ─── END EXAM ─────────────────────────────────────────────
    {
        let state_for_end = state.clone();
        let window_for_end = window.clone();
        let app_for_end = app.clone();
        let start_btn_clone = start_btn.clone();
        let end_btn_clone = end_btn.clone();
        let exam_window_for_end = exam_window_handle.clone();

        end_btn.connect_clicked(move |_| {
            show_end_exam_dialog(
                &window_for_end,
                &app_for_end,
                state_for_end.clone(),
                start_btn_clone.clone(),
                end_btn_clone.clone(),
                exam_window_for_end.clone(),
            );
        });
    }

    window.present();
}

// ─── END EXAM DIALOG ──────────────────────────────────────────────────────────

fn show_end_exam_dialog(
    home_window: &ApplicationWindow,
    app: &Application,
    state: Rc<RefCell<AppState>>,
    start_btn: Button,
    end_btn: Button,
    exam_window_handle: Rc<RefCell<Option<ApplicationWindow>>>,
) {
    let dialog = MessageDialog::builder()
        .transient_for(home_window)
        .heading("End Exam?")
        .body("Export the report before ending. Once you confirm the export the exam will end and all data will be reset.")
        .build();

    dialog.add_response("cancel", "Cancel");
    dialog.add_response("export_end", "Export & End");
    dialog.set_default_response(Some("cancel"));
    dialog.set_response_appearance("export_end", ResponseAppearance::Destructive);

    let home_window = home_window.clone();
    let app = app.clone();

    dialog.connect_response(None, move |dlg, response| {
        dlg.close();
        if response != "export_end" {
            return;
        }

        // Log ExamEnded before generating the report
        {
            let mut s = state.borrow_mut();
            if !s.exam_ended {
                s.exam_ended = true;
                s.log_event(EventKind::ExamEnded);
            }
        }

        // Mark exam as ended — backup timer will now stop on next tick
        let backup_path = get_backup_path();

        let md = export_to_markdown(&state.borrow());
        let state_for_save = state.clone();
        let home_window_for_save = home_window.clone();
        let app_for_save = app.clone();
        let start_btn_for_save = start_btn.clone();
        let end_btn_for_save = end_btn.clone();
        let exam_win_for_save = exam_window_handle.clone();

        // on_saved callback — only runs after a successful file write
        let on_saved = move || {
            // Close student-facing window
            if let Some(ew) = exam_win_for_save.borrow().as_ref() {
                ew.close();
            }
            *exam_win_for_save.borrow_mut() = None;

            // Reset all state
            state_for_save.borrow_mut().reset();

            // Swap buttons back
            end_btn_for_save.set_visible(false);
            start_btn_for_save.set_visible(true);

            // Close the current home window and reopen fresh
            home_window_for_save.close();
            crate::ui::home::build(&app_for_save, state_for_save.clone());
        };

        open_save_dialog(&home_window, md, Some(Rc::new(on_saved)), Some(backup_path));
    });

    dialog.present();
}

// ─── HELPERS ──────────────────────────────────────────────────────────────────

fn update_stack_page(stack: &Stack, state: &AppState) {
    if state.students.is_empty() {
        stack.set_visible_child_name("empty");
    } else {
        stack.set_visible_child_name("students");
    }
}

fn validate_start(s: &AppState) -> Option<&'static str> {
    let name = s.exam.name.trim();
    let prof = s.exam.professor.trim();
    if name.is_empty() && prof.is_empty() {
        Some("Please enter the exam name and professor name.")
    } else if name.is_empty() {
        Some("Please enter the exam name.")
    } else if prof.is_empty() {
        Some("Please enter the professor name.")
    } else if s.students.is_empty() {
        Some("Please import a student list before starting.")
    } else {
        None
    }
}

fn show_error_dialog(window: &ApplicationWindow, msg: &str) {
    let dialog = MessageDialog::builder()
        .transient_for(window)
        .heading("Cannot Start Exam")
        .body(msg)
        .build();
    dialog.add_response("ok", "OK");
    dialog.set_default_response(Some("ok"));
    dialog.set_response_appearance("ok", ResponseAppearance::Suggested);
    dialog.present();
}

/// Open a native save-file dialog pre-pointed at the backup folder on the Desktop.
/// If the user confirms, the file is written there — replacing the auto-backup.
/// `on_saved` is called only after a successful file write.
/// When `on_saved` is None (mid-exam export) nothing extra happens after saving.
fn open_save_dialog(
    window: &ApplicationWindow,
    md: String,
    on_saved: Option<Rc<dyn Fn()>>,
    initial_path: Option<std::path::PathBuf>, // pre-fills the save dialog location
) {
    let mut dialog_builder = FileDialog::builder()
        .title("Export Exam Report")
        .initial_name("exam_report.md");

    // Pre-open the dialog at the backup folder on Desktop
    if let Some(ref path) = initial_path {
        if let Some(parent) = path.parent() {
            let folder = gtk::gio::File::for_path(parent);
            dialog_builder = dialog_builder.initial_folder(&folder);
        }
    }

    let dialog = dialog_builder.build();

    let window_ref = window.clone();
    dialog.save(
        Some(&window.clone()),
        None::<&gio::Cancellable>,
        move |result| {
            let file = match result {
                Ok(f) => f,
                Err(_) => return, // user cancelled — exam keeps running
            };
            let path = match file.path() {
                Some(p) => p,
                None => {
                    show_save_error(&window_ref, "Could not determine the save path.");
                    return;
                }
            };
            if let Err(e) = std::fs::write(&path, md.as_bytes()) {
                show_save_error(&window_ref, &format!("Failed to write file:\n{e}"));
                return;
            }
            // File written successfully — trigger reset if this was an End Exam export
            if let Some(cb) = &on_saved {
                cb();
            }
        },
    );
}

fn show_save_error(window: &ApplicationWindow, msg: &str) {
    let dialog = MessageDialog::builder()
        .transient_for(window)
        .heading("Export Failed")
        .body(msg)
        .build();
    dialog.add_response("ok", "OK");
    dialog.set_default_response(Some("ok"));
    dialog.present();
}
