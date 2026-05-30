use std::cell::{Cell, RefCell};
use std::rc::Rc;
use adw::prelude::*;
use gtk::{Box, Button, CheckButton, Entry, Label, Orientation};
use crate::models::app_state::AppState;
use crate::models::exam_event::EventKind;


/// Returns (card_widget, on_student_selected_callback)
pub fn build(state: Rc<RefCell<AppState>>) -> (Box, impl Fn(Option<usize>) + 'static) {
    let name_label = Label::builder()
        .label("Select a student")
        .halign(gtk::Align::Start)
        .css_classes(["title-4"])
        .wrap(true)
        .build();

    let meta_label = Label::builder()
        .halign(gtk::Align::Start)
        .css_classes(["dim-label"])
        .wrap(true)
        .build();

    let restroom_toggle = CheckButton::builder()
        .label("In Restroom")
        .sensitive(false)
        .build();

    let notes_title = Label::builder()
        .label("Student Notes")
        .halign(gtk::Align::Start)
        .css_classes(["title-4"])
        .margin_top(8)
        .margin_bottom(8)
        .build();

    let note_entry = Entry::builder()
        .placeholder_text("Add a note for this student...")
        .hexpand(true)
        .build();

    let add_btn = Button::builder()
        .label("Add")
        .css_classes(["suggested-action", "pill"])
        .sensitive(false)
        .build();

    let input_row = Box::builder()
        .orientation(Orientation::Horizontal)
        .spacing(8)
        .build();
    input_row.append(&note_entry);
    input_row.append(&add_btn);

    let notes_list = Box::builder()
        .orientation(Orientation::Vertical)
        .spacing(6)
        .margin_top(8)
        .build();

    let inner = Box::builder()
        .orientation(Orientation::Vertical)
        .spacing(8)
        .margin_top(24).margin_bottom(24)
        .margin_start(24).margin_end(24)
        .build();

    let panel_title = Label::builder()
        .label("Student Details")
        .halign(gtk::Align::Start)
        .css_classes(["title-4"])
        .margin_bottom(12)
        .build();

    inner.append(&panel_title);
    inner.append(&name_label);
    inner.append(&meta_label);
    inner.append(&restroom_toggle);
    inner.append(&notes_title);
    inner.append(&input_row);
    inner.append(&notes_list);

    let card = super::exam_details_card::wrap_in_card(inner);

    // ── Shared selected index ─────────────────────────────────
    let selected: Rc<RefCell<Option<usize>>> = Rc::new(RefCell::new(None));
    let updating = Rc::new(Cell::new(false));

    // ── refresh_details closure ───────────────────────────────
    let refresh = {
        let state          = state.clone();
        let selected       = selected.clone();
        let name_label     = name_label.clone();
        let meta_label     = meta_label.clone();
        let restroom_toggle = restroom_toggle.clone();
        let notes_list     = notes_list.clone();
        let note_entry     = note_entry.clone();
        let add_btn        = add_btn.clone();
        let updating       = updating.clone();
        Rc::new(move || {
            updating.set(true);
            let s = state.borrow();
            if let Some(idx) = *selected.borrow() {
                if let Some(student) = s.students.get(idx) {
                    name_label.set_text(&student.name);
                    meta_label.set_text(&format!(
                        "{} · {}", student.matriculation_number, student.birthdate
                    ));
                    restroom_toggle.set_sensitive(true);
                    restroom_toggle.set_active(student.in_restroom);
                    while let Some(c) = notes_list.first_child() { notes_list.remove(&c); }
                    for note in &student.notes {
                        append_student_note_row(&notes_list, note, state.clone(), idx);
                    }
                    note_entry.set_text("");
                    add_btn.set_sensitive(true);
                    updating.set(false);
                    return;
                }
            }
            name_label.set_text("Select a student");
            meta_label.set_text("Student details will appear here.");
            restroom_toggle.set_active(false);
            restroom_toggle.set_sensitive(false);
            while let Some(c) = notes_list.first_child() { notes_list.remove(&c); }
            note_entry.set_text("");
            add_btn.set_sensitive(false);
            updating.set(false);
        })
    };

    // ── Restroom toggle ───────────────────────────────────────
    {
        let state    = state.clone();
        let selected = selected.clone();
        let refresh  = refresh.clone();
        let updating = updating.clone();
        restroom_toggle.connect_toggled(move |toggle| {
            if updating.get() { return; }
            if let Some(idx) = *selected.borrow() {
                let mut s = state.borrow_mut();
                if let Some(student) = s.students.get_mut(idx) {
                    let is_in = toggle.is_active();
                    student.in_restroom = is_in;
                    let event = if is_in {
                        EventKind::StudentEnteredRestroom {
                            name: student.name.clone(),
                            matriculation_number: student.matriculation_number.clone(),
                        }
                    } else {
                        EventKind::StudentLeftRestroom {
                            name: student.name.clone(),
                            matriculation_number: student.matriculation_number.clone(),
                        }
                    };
                    s.log_event(event);
                }
            }
            refresh();
        });
    }

    // ── Add note ──────────────────────────────────────────────
    {
        let state     = state.clone();
        let selected  = selected.clone();
        let list      = notes_list.clone();
        let entry_ref = note_entry.clone();
        let add = Rc::new(move || {
            let text = entry_ref.text().trim().to_string();
            if text.is_empty() { return; }
            if let Some(idx) = *selected.borrow() {
                {
                    let mut s = state.borrow_mut();
                    if let Some(student) = s.students.get_mut(idx) {
                        student.notes.push(text.clone());
                        // Clone what we need before calling log_event to avoid double-borrow
                        let name = student.name.clone();
                        let matno = student.matriculation_number.clone();
                        let _ = student;
                        s.log_event(EventKind::StudentNoteAdded {
                            name,
                            matriculation_number: matno,
                            note: text.clone(),
                        });
                    }
                }
                append_student_note_row(&list, &text, state.clone(), idx);
                entry_ref.set_text("");
            }
        });
        let add_btn_cb  = add.clone();
        add_btn.connect_clicked(move |_| add_btn_cb());
        let add_entry_cb = add.clone();
        note_entry.connect_activate(move |_| add_entry_cb());
    }

    // ── on_selected callback returned to home.rs ──────────────
    let on_selected = {
        let selected = selected.clone();
        let refresh  = refresh.clone();
        move |index: Option<usize>| {
            *selected.borrow_mut() = index;
            refresh();
        }
    };

    (card, on_selected)
}

fn append_student_note_row(
    list: &Box,
    text: &str,
    state: Rc<RefCell<AppState>>,
    student_index: usize,
) {
    let row = Box::builder()
        .orientation(Orientation::Horizontal)
        .spacing(8)
        .build();

    let label = Label::builder()
        .label(text)
        .halign(gtk::Align::Start)
        .hexpand(true)
        .wrap(true)
        .wrap_mode(gtk::pango::WrapMode::Word)
        .build();

    let del = Button::builder()
        .icon_name("user-trash-symbolic")
        .css_classes(["flat", "circular"])
        .valign(gtk::Align::Center)
        .build();

    row.append(&label);
    row.append(&del);
    list.append(&row);

    let row_ref  = row.clone();
    let text_own = text.to_string();
    del.connect_clicked(move |_| {
        let mut s = state.borrow_mut();
        if let Some(student) = s.students.get_mut(student_index) {
            student.notes.retain(|n| n != &text_own);
        }
        if let Some(p) = row_ref.parent() {
            if let Ok(b) = p.downcast::<Box>() { b.remove(&row_ref); }
        }
    });
}
