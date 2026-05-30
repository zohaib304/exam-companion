use std::cell::RefCell;
use std::rc::Rc;

use adw::prelude::*;
use adw::{Application, ApplicationWindow, HeaderBar, MessageDialog, ResponseAppearance, StatusPage};
use gtk::{Box, Button, Orientation, Separator, Stack};

use crate::models::app_state::AppState;
use crate::ui::{import, student_list, student_status_message};
use crate::ui::components::{exam_details_card, timer_card, notes_card, student_details};

pub fn build(app: &Application, state: Rc<RefCell<AppState>>) {
    // ─── HEADER ───────────────────────────────────────────────
    let header = HeaderBar::new();
    header.set_decoration_layout(Some("icon:minimize,maximize,close"));

    let start_btn = Button::builder()
        .label("Save & Start")
        .css_classes(["suggested-action", "pill"])
        .build();
    header.pack_end(&start_btn);

    // ─── LEFT COLUMN ──────────────────────────────────────────
    let screen_title = gtk::Label::builder()
        .label("Setup")
        .halign(gtk::Align::Start)
        .css_classes(["title-2"])
        .margin_bottom(4)
        .build();

    let left_card  = exam_details_card::build(state.clone());
    let t_card     = timer_card::build(state.clone());
    let n_card     = notes_card::build(state.clone());
    let duration_mins = timer_card::duration_mins(state.clone());

    let left_column = Box::builder()
        .orientation(Orientation::Vertical)
        .spacing(12)
        .margin_top(24).margin_start(24).margin_end(12).margin_bottom(24)
        .width_request(180)
        .hexpand(true).vexpand(true)
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
    let (details_card, on_student_selected) = student_details::build(state.clone());

    let student_panel = student_list::StudentListPanel::new(
        state.clone(),
        Rc::new(move |index| on_student_selected(index)),
    );

    let stack = Stack::builder().vexpand(true).hexpand(true).build();
    stack.add_named(&status_page, Some("empty"));
    stack.add_named(student_panel.widget(), Some("students"));
    update_stack_page(&stack, &state.borrow());

    let middle_column = Box::builder()
        .orientation(Orientation::Vertical)
        .spacing(12)
        .margin_top(24).margin_start(12).margin_end(12).margin_bottom(24)
        .width_request(360)
        .hexpand(true).vexpand(true)
        .build();
    middle_column.append(&stack);

    let right_column = Box::builder()
        .orientation(Orientation::Vertical)
        .spacing(12)
        .margin_top(24).margin_start(12).margin_end(24).margin_bottom(24)
        .width_request(360)
        .hexpand(true).vexpand(true)
        .build();
    right_column.append(&details_card);

    // ─── LAYOUT ───────────────────────────────────────────────
    let sep1 = Separator::new(Orientation::Vertical);
    let sep2 = Separator::new(Orientation::Vertical);

    let columns = Box::builder()
        .orientation(Orientation::Horizontal)
        .hexpand(true).vexpand(true)
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
    let state_for_import  = state.clone();
    let on_imported_btn   = on_imported.clone();
    import_btn.connect_clicked(move |_| {
        import::open_csv_import_dialog(
            &window_for_import,
            state_for_import.clone(),
            on_imported_btn.clone(),
        );
    });

    // ─── SAVE & START ─────────────────────────────────────────
    let state_for_start  = state.clone();
    let window_for_start = window.clone();
    let app_for_start    = app.clone();

    start_btn.connect_clicked(move |_| {
        let s = state_for_start.borrow();
        let error_msg = validate_start(&s);
        drop(s);

        if let Some(msg) = error_msg {
            show_error_dialog(&window_for_start, msg);
            return;
        }

        let mins = *duration_mins.borrow();
        state_for_start.borrow_mut().exam.duration_secs = mins * 60;
        crate::ui::exam_window::open(&app_for_start, state_for_start.clone());
    });

    window.present();
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