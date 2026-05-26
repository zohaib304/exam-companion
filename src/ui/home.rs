use std::cell::{Cell, RefCell};
use std::rc::Rc;

use adw::prelude::*;
use adw::{
    Application, ApplicationWindow, HeaderBar, MessageDialog, ResponseAppearance, StatusPage,
};
use gtk::{Box, Button, CheckButton, Entry, Label, Orientation, Separator, Stack};

use crate::models::app_state::AppState;
use crate::ui::{import, student_list, student_status_message};

pub fn build(app: &Application, state: Rc<RefCell<AppState>>) {
    // ─── HEADER BAR ───────────────────────────────────────────
    let header: HeaderBar = HeaderBar::new();
    header.set_decoration_layout(Some("icon:minimize,maximize,close"));

    let start_btn: Button = Button::builder()
        .label("Save & Start")
        .css_classes(["suggested-action", "pill"])
        .build();
    header.pack_end(&start_btn);

    // ─── LEFT COLUMN ──────────────────────────────────────────
    let screen_title = Label::builder()
        .label("Setup")
        .halign(gtk::Align::Start)
        .css_classes(["title-2"])
        .margin_bottom(4)
        .build();

    // ── Exam Details Card ─────────────────────────────────────
    let left_title = Label::builder()
        .label("Exam Details")
        .halign(gtk::Align::Start)
        .css_classes(["title-4"])
        .margin_bottom(12)
        .build();

    let exam_name_label = Label::builder()
        .label("Exam Name")
        .halign(gtk::Align::Start)
        .css_classes(["caption"])
        .build();

    let exam_name_entry = Entry::builder()
        .placeholder_text("e.g. Mathematics Final")
        .hexpand(true)
        .build();

    let professor_label = Label::builder()
        .label("Professor Name")
        .halign(gtk::Align::Start)
        .css_classes(["caption"])
        .margin_top(12)
        .build();

    let professor_entry = Entry::builder()
        .placeholder_text("e.g. Prof. Schmidt")
        .hexpand(true)
        .build();

    // Populate from state
    {
        let s = state.borrow();
        exam_name_entry.set_text(&s.exam.name);
        professor_entry.set_text(&s.exam.professor);
    }

    let state_for_exam = state.clone();
    exam_name_entry.connect_changed(move |entry| {
        state_for_exam.borrow_mut().exam.name = entry.text().to_string();
    });

    let state_for_prof = state.clone();
    professor_entry.connect_changed(move |entry| {
        state_for_prof.borrow_mut().exam.professor = entry.text().to_string();
    });

    let left_card_inner = Box::builder()
        .orientation(Orientation::Vertical)
        .spacing(4)
        .margin_top(24)
        .margin_bottom(24)
        .margin_start(24)
        .margin_end(24)
        .build();

    left_card_inner.append(&left_title);
    left_card_inner.append(&exam_name_label);
    left_card_inner.append(&exam_name_entry);
    left_card_inner.append(&professor_label);
    left_card_inner.append(&professor_entry);

    let left_card = Box::builder()
        .orientation(Orientation::Vertical)
        .css_classes(["card"])
        .hexpand(true)
        .vexpand(false)
        .build();
    left_card.append(&left_card_inner);

    // ── Timer Card ────────────────────────────────────────────
    let timer_title = Label::builder()
        .label("Exam Duration")
        .halign(gtk::Align::Start)
        .css_classes(["title-4"])
        .margin_bottom(12)
        .build();

    let duration_mins = Rc::new(RefCell::new(state.borrow().exam.duration_secs / 60));

    let duration_label = Label::builder()
        .label(&format!("{} min", duration_mins.borrow()))
        .css_classes(["title-1"])
        .halign(gtk::Align::Center)
        .hexpand(true)
        .build();

    let minus_btn = Button::builder()
        .label("−")
        .css_classes(["circular", "flat"])
        .sensitive(false)
        .build();

    let plus_btn = Button::builder()
        .label("+")
        .css_classes(["circular", "flat"])
        .build();

    let timer_controls = Box::builder()
        .orientation(Orientation::Horizontal)
        .spacing(12)
        .halign(gtk::Align::Center)
        .valign(gtk::Align::Center)
        .margin_top(8)
        .build();

    timer_controls.append(&minus_btn);
    timer_controls.append(&duration_label);
    timer_controls.append(&plus_btn);

    let step_label = Label::builder()
        .label("Step: 5 min  |  Minimum: 90 min")
        .halign(gtk::Align::Center)
        .css_classes(["caption", "dim-label"])
        .margin_top(4)
        .build();

    let duration_for_minus = duration_mins.clone();
    let label_for_minus = duration_label.clone();
    let minus_btn_ref = minus_btn.clone();
    let state_for_minus = state.clone();
    minus_btn.connect_clicked(move |_| {
        let mut mins = duration_for_minus.borrow_mut();
        if *mins > 90 {
            *mins -= 5;
            label_for_minus.set_text(&format!("{} min", *mins));
            state_for_minus.borrow_mut().exam.duration_secs = *mins * 60;
        }
        if *mins <= 90 {
            minus_btn_ref.set_sensitive(false);
        }
    });

    let duration_for_plus = duration_mins.clone();
    let label_for_plus = duration_label.clone();
    let minus_btn_for_plus = minus_btn.clone();
    let state_for_timer = state.clone();
    plus_btn.connect_clicked(move |_| {
        let mut mins = duration_for_plus.borrow_mut();
        *mins += 5;
        label_for_plus.set_text(&format!("{} min", *mins));
        state_for_timer.borrow_mut().exam.duration_secs = *mins * 60;
        minus_btn_for_plus.set_sensitive(true);
    });

    let timer_card_inner = Box::builder()
        .orientation(Orientation::Vertical)
        .spacing(4)
        .margin_top(24)
        .margin_bottom(24)
        .margin_start(24)
        .margin_end(24)
        .build();

    timer_card_inner.append(&timer_title);
    timer_card_inner.append(&timer_controls);
    timer_card_inner.append(&step_label);

    let timer_card = Box::builder()
        .orientation(Orientation::Vertical)
        .css_classes(["card"])
        .hexpand(true)
        .vexpand(false)
        .build();
    timer_card.append(&timer_card_inner);

    // ── Notes Card ────────────────────────────────────────────
    let notes_title = Label::builder()
        .label("Exam Notes")
        .halign(gtk::Align::Start)
        .css_classes(["title-4"])
        .margin_bottom(12)
        .build();

    let notes_hint = Label::builder()
        .label("Each saved note appears instantly on the exam window.")
        .halign(gtk::Align::Start)
        .css_classes(["caption", "dim-label"])
        .margin_bottom(8)
        .build();

    let notes_card_inner = Box::builder()
        .orientation(Orientation::Vertical)
        .spacing(4)
        .margin_top(24)
        .margin_bottom(24)
        .margin_start(24)
        .margin_end(24)
        .build();

    let notes_entry = Entry::builder()
        .placeholder_text("Type a note and press Add...")
        .hexpand(true)
        .build();

    let add_note_btn = Button::builder()
        .label("Add")
        .css_classes(["suggested-action", "pill"])
        .build();

    let input_row = Box::builder()
        .orientation(Orientation::Horizontal)
        .spacing(8)
        .build();

    input_row.append(&notes_entry);
    input_row.append(&add_note_btn);

    // Notes list box — rows appear here as professor adds notes
    let notes_list = Box::builder()
        .orientation(Orientation::Vertical)
        .spacing(6)
        .margin_top(8)
        .build();

    // Populate existing notes from state on load
    {
        let s = state.borrow();
        for note in &s.exam.notes {
            append_note_row(&notes_list, note, state.clone());
        }
    }

    notes_card_inner.append(&notes_title);
    notes_card_inner.append(&notes_hint);
    notes_card_inner.append(&input_row);
    notes_card_inner.append(&notes_list);

    let notes_card = Box::builder()
        .orientation(Orientation::Vertical)
        .css_classes(["card"])
        .hexpand(true)
        .vexpand(false)
        .build();
    notes_card.append(&notes_card_inner);

    // Shared add action (used by both button and Enter key)
    let state_for_add = state.clone();
    let notes_list_for_add = notes_list.clone();
    let notes_entry_for_add = notes_entry.clone();
    let add_note = Rc::new(move || {
        let text = notes_entry_for_add.text().trim().to_string();
        if text.is_empty() {
            return;
        }
        state_for_add.borrow_mut().exam.notes.push(text.clone());
        append_note_row(&notes_list_for_add, &text, state_for_add.clone());
        notes_entry_for_add.set_text("");
    });

    let add_note_for_btn = add_note.clone();
    add_note_btn.connect_clicked(move |_| {
        add_note_for_btn();
    });

    let add_note_for_entry = add_note.clone();
    notes_entry.connect_activate(move |_| {
        // fires on Enter key
        add_note_for_entry();
    });

    // ── Left Column ───────────────────────────────────────────
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
    left_column.append(&timer_card);
    left_column.append(&notes_card);

    // ─── MIDDLE COLUMN — Students ─────────────────────────────
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

    // ─── RIGHT COLUMN — Details ───────────────────────────────
    let selected_student_name = Label::builder()
        .label("Select a student")
        .halign(gtk::Align::Start)
        .css_classes(["title-4"])
        .wrap(true)
        .build();

    let selected_student_meta = Label::builder()
        .halign(gtk::Align::Start)
        .css_classes(["dim-label"])
        .wrap(true)
        .build();

    let restroom_toggle = CheckButton::builder().label("Restroom").build();

    restroom_toggle.set_sensitive(false);

    let selected_student_state = Rc::new(RefCell::new(None::<usize>));
    let updating_details = Rc::new(Cell::new(false));

    let details_title = Label::builder()
        .label("Details Panel")
        .halign(gtk::Align::Start)
        .css_classes(["title-4"])
        .margin_bottom(12)
        .build();

    let details_card_inner = Box::builder()
        .orientation(Orientation::Vertical)
        .spacing(8)
        .margin_top(24)
        .margin_bottom(24)
        .margin_start(24)
        .margin_end(24)
        .build();

    details_card_inner.append(&details_title);
    details_card_inner.append(&selected_student_name);
    details_card_inner.append(&selected_student_meta);
    details_card_inner.append(&restroom_toggle);

    // ── Student Notes (per-student) ─────────────────────────
    let student_notes_title = Label::builder()
        .label("Student Notes")
        .halign(gtk::Align::Start)
        .css_classes(["title-4"])
        .margin_top(8)
        .margin_bottom(8)
        .build();

    let student_note_entry = Entry::builder()
        .placeholder_text("Add a note for this student...")
        .hexpand(true)
        .build();

    let student_add_note_btn = Button::builder()
        .label("Add")
        .css_classes(["suggested-action", "pill"])
        .build();

    let student_input_row = Box::builder()
        .orientation(Orientation::Horizontal)
        .spacing(8)
        .build();
    student_input_row.append(&student_note_entry);
    student_input_row.append(&student_add_note_btn);

    let student_notes_list = Box::builder()
        .orientation(Orientation::Vertical)
        .spacing(6)
        .margin_top(8)
        .build();

    details_card_inner.append(&student_notes_title);
    details_card_inner.append(&student_input_row);
    details_card_inner.append(&student_notes_list);

    let details_card = Box::builder()
        .orientation(Orientation::Vertical)
        .css_classes(["card"])
        .width_request(360)
        .hexpand(true)
        .vexpand(false)
        .build();
    details_card.append(&details_card_inner);

    let refresh_details = {
        let state = state.clone();
        let selected_student_state = selected_student_state.clone();
        let selected_student_name = selected_student_name.clone();
        let selected_student_meta = selected_student_meta.clone();
        let restroom_toggle = restroom_toggle.clone();
        let student_notes_list = student_notes_list.clone();
        let student_note_entry = student_note_entry.clone();
        let student_add_note_btn = student_add_note_btn.clone();
        let updating_details = updating_details.clone();
        Rc::new(move || {
            updating_details.set(true);
            let s = state.borrow();
            let selected_index = *selected_student_state.borrow();

            if let Some(index) = selected_index {
                if let Some(student) = s.students.get(index) {
                    selected_student_name.set_text(&student.name);
                    selected_student_meta.set_text(&format!(
                        "{} · {}",
                        student.matriculation_number, student.birthdate
                    ));
                    restroom_toggle.set_sensitive(true);
                    restroom_toggle.set_active(student.in_restroom);
                    // populate student notes
                    // clear existing UI rows
                    if let Some(parent) = student_notes_list.parent() {
                        // nothing to do; student_notes_list persists
                    }
                    // remove all children from notes list
                    while let Some(child) = student_notes_list.first_child() {
                        student_notes_list.remove(&child);
                    }
                    for (ni, note) in student.notes.iter().enumerate() {
                        append_student_note_row(&student_notes_list, note, state.clone(), index, ni);
                    }
                    student_note_entry.set_text("");
                    student_add_note_btn.set_sensitive(true);
                    updating_details.set(false);
                    return;
                }
            }

            selected_student_name.set_text("Select a student");
            selected_student_meta.set_text("Student details will appear here.");
            restroom_toggle.set_active(false);
            restroom_toggle.set_sensitive(false);
            // clear student notes UI when no selection
            while let Some(child) = student_notes_list.first_child() {
                student_notes_list.remove(&child);
            }
            student_note_entry.set_text("");
            student_add_note_btn.set_sensitive(false);
            updating_details.set(false);
        })
    };

    {
        let selected_student_state = selected_student_state.clone();
        let state = state.clone();
        let refresh_details = refresh_details.clone();
        let updating_details = updating_details.clone();
        restroom_toggle.connect_toggled(move |toggle| {
            if updating_details.get() {
                return;
            }
            if let Some(index) = *selected_student_state.borrow() {
                if let Some(student) = state.borrow_mut().students.get_mut(index) {
                    student.in_restroom = toggle.is_active();
                }
            }
            refresh_details();
        });
    }

    // Student add-note action
    {
        let selected_student_state = selected_student_state.clone();
        let state = state.clone();
        let student_notes_list = student_notes_list.clone();
        let student_note_entry_for_add = student_note_entry.clone();
        let add_closure = Rc::new(move || {
            let text = student_note_entry_for_add.text().trim().to_string();
            if text.is_empty() {
                return;
            }
            if let Some(idx) = *selected_student_state.borrow() {
                state.borrow_mut().students.get_mut(idx).map(|s| s.notes.push(text.clone()));
                append_student_note_row(&student_notes_list, &text, state.clone(), idx, std::usize::MAX);
                student_note_entry_for_add.set_text("");
            }
        });

        let add_for_btn = add_closure.clone();
        student_add_note_btn.connect_clicked(move |_| {
            add_for_btn();
        });

        let add_for_entry = add_closure.clone();
        student_note_entry.connect_activate(move |_| {
            add_for_entry();
        });
    }

    let selected_student_state_for_list = selected_student_state.clone();
    let refresh_details_for_list = refresh_details.clone();
    let student_panel = student_list::StudentListPanel::new(
        state.clone(),
        Rc::new(move |index| {
            *selected_student_state_for_list.borrow_mut() = index;
            refresh_details_for_list();
        }),
    );

    refresh_details();

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

    // ─── THREE COLUMN LAYOUT ─────────────────────────────────
    let separator = Separator::new(Orientation::Vertical);

    let columns = Box::builder()
        .orientation(Orientation::Horizontal)
        .hexpand(true)
        .vexpand(true)
        .build();

    columns.append(&left_column);
    columns.append(&separator);
    columns.append(&middle_column);
    columns.append(&separator);
    columns.append(&right_column);

    // ─── ROOT LAYOUT ──────────────────────────────────────────
    let content = Box::builder().orientation(Orientation::Vertical).build();

    content.append(&header);
    content.append(&columns);

    // ─── WINDOW ───────────────────────────────────────────────
    let window = ApplicationWindow::builder()
        .application(app)
        .title("Exam Companion")
        .default_width(900)
        .default_height(600)
        .resizable(true)
        .content(&content)
        .build();

    // ─── IMPORT CALLBACKS ─────────────────────────────────────
    let on_imported = {
        let stack = stack.clone();
        let student_panel = student_panel.clone();
        let state = state.clone();
        Rc::new(move || {
            update_stack_page(&stack, &state.borrow());
            student_panel.refresh();
        })
    };

    let state_for_import = state.clone();
    let window_for_import = window.clone();
    let on_imported_for_btn = on_imported.clone();

    let connect_import = |button: &Button| {
        let window = window_for_import.clone();
        let state = state_for_import.clone();
        let on_imported = on_imported_for_btn.clone();
        button.connect_clicked(move |_| {
            import::open_csv_import_dialog(&window, state.clone(), on_imported.clone());
        });
    };

    connect_import(&import_btn);

    // ─── SAVE & START ─────────────────────────────────────────
    let state_for_start = state.clone();
    let window_for_start = window.clone();
    let app_for_start = app.clone();
    let duration_for_start = duration_mins.clone();

    start_btn.connect_clicked(move |_| {
        let s = state_for_start.borrow();
        let exam_name = s.exam.name.trim().to_string();
        let professor = s.exam.professor.trim().to_string();
        let has_students = !s.students.is_empty();
        drop(s);

        // ── Validation ────────────────────────────────────────
        let error_msg = if exam_name.is_empty() && professor.is_empty() {
            Some("Please enter the exam name and professor name.")
        } else if exam_name.is_empty() {
            Some("Please enter the exam name.")
        } else if professor.is_empty() {
            Some("Please enter the professor name.")
        } else if !has_students {
            Some("Please import a student list before starting.")
        } else {
            None
        };

        if let Some(msg) = error_msg {
            let dialog = MessageDialog::builder()
                .transient_for(&window_for_start)
                .heading("Cannot Start Exam")
                .body(msg)
                .build();

            dialog.add_response("ok", "OK");
            dialog.set_default_response(Some("ok"));
            dialog.set_response_appearance("ok", ResponseAppearance::Suggested);
            dialog.present();
            return;
        }

        // ── Save duration to state ────────────────────────────
        let mins = *duration_for_start.borrow();
        state_for_start.borrow_mut().exam.duration_secs = mins * 60;

        // ── Open Exam Window ──────────────────────────────────
        crate::ui::exam_window::open(&app_for_start, state_for_start.clone());
    });

    window.present();
}

fn update_stack_page(stack: &Stack, state: &AppState) {
    if state.students.is_empty() {
        stack.set_visible_child_name("empty");
    } else {
        stack.set_visible_child_name("students");
    }
}

fn append_note_row(list: &Box, text: &str, state: Rc<RefCell<AppState>>) {
    let row = Box::builder()
        .orientation(Orientation::Horizontal)
        .spacing(8)
        .build();

    let note_label = Label::builder()
        .label(text)
        .halign(gtk::Align::Start)
        .hexpand(true)
        .wrap(true)
        .wrap_mode(gtk::pango::WrapMode::Word)
        .build();

    let remove_btn = Button::builder()
        .icon_name("user-trash-symbolic")
        .css_classes(["flat", "circular"])
        .valign(gtk::Align::Center)
        .build();

    row.append(&note_label);
    row.append(&remove_btn);
    list.append(&row);

    let row_ref = row.clone();
    let text_owned = text.to_string();
    remove_btn.connect_clicked(move |_| {
        state.borrow_mut().exam.notes.retain(|n| n != &text_owned);
        if let Some(parent) = row_ref.parent() {
            if let Ok(b) = parent.downcast::<Box>() {
                b.remove(&row_ref);
            }
        }
    });
}

fn append_student_note_row(
    list: &Box,
    text: &str,
    state: Rc<RefCell<AppState>>,
    student_index: usize,
    note_index: usize,
) {
    let row = Box::builder()
        .orientation(Orientation::Horizontal)
        .spacing(8)
        .build();

    let note_label = Label::builder()
        .label(text)
        .halign(gtk::Align::Start)
        .hexpand(true)
        .wrap(true)
        .wrap_mode(gtk::pango::WrapMode::Word)
        .build();

    let remove_btn = Button::builder()
        .icon_name("user-trash-symbolic")
        .css_classes(["flat", "circular"])
        .valign(gtk::Align::Center)
        .build();

    row.append(&note_label);
    row.append(&remove_btn);
    list.append(&row);

    let row_ref = row.clone();
    let text_owned = text.to_string();
    remove_btn.connect_clicked(move |_| {
        let mut s = state.borrow_mut();
        if let Some(student) = s.students.get_mut(student_index) {
            if note_index != std::usize::MAX {
                if note_index < student.notes.len() {
                    student.notes.remove(note_index);
                } else {
                    student.notes.retain(|n| n != &text_owned);
                }
            } else {
                student.notes.retain(|n| n != &text_owned);
            }
        }
        if let Some(parent) = row_ref.parent() {
            if let Ok(b) = parent.downcast::<Box>() {
                b.remove(&row_ref);
            }
        }
    });
}