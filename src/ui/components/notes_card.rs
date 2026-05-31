use std::cell::RefCell;
use std::rc::Rc;
use adw::prelude::*;
use gtk::{Box, Button, Entry, Label, Orientation, ScrolledWindow};
use crate::models::app_state::AppState;
use crate::models::exam_event::EventKind;

pub fn build(state: Rc<RefCell<AppState>>) -> Box {
    let title = Label::builder()
        .label("Exam Notes")
        .halign(gtk::Align::Start)
        .css_classes(["title-4"])
        .margin_bottom(12)
        .build();

    let hint = Label::builder()
        .label("Each saved note appears instantly on the exam window.")
        .halign(gtk::Align::Start)
        .css_classes(["caption", "dim-label"])
        .margin_bottom(8)
        .build();

    let entry = Entry::builder()
        .placeholder_text("Type a note and press Add...")
        .hexpand(true)
        .build();

    let add_btn = Button::builder()
        .label("Add")
        .css_classes(["suggested-action", "pill"])
        .build();

    let input_row = Box::builder()
        .orientation(Orientation::Horizontal)
        .spacing(8)
        .build();
    input_row.append(&entry);
    input_row.append(&add_btn);

    let list = Box::builder()
        .orientation(Orientation::Vertical)
        .spacing(6)
        .margin_top(8)
        .build();

    let scroll = ScrolledWindow::builder()
        .min_content_height(150)
        .vexpand(true)
        .child(&list)
        .build();

    {
        let s = state.borrow();
        for note in &s.exam.notes {
            append_note_row(&list, note, state.clone());
        }
    }

    let add = {
        let state = state.clone();
        let list  = list.clone();
        let entry = entry.clone();
        Rc::new(move || {
            let text = entry.text().trim().to_string();
            if text.is_empty() { return; }
            {
                let mut s = state.borrow_mut();
                s.exam.notes.push(text.clone());
                s.log_event(EventKind::ExamNoteAdded { note: text.clone() });
            }
            append_note_row(&list, &text, state.clone());
            entry.set_text("");
        })
    };

    let add_btn_cb = add.clone();
    add_btn.connect_clicked(move |_| add_btn_cb());
    let add_entry_cb = add.clone();
    entry.connect_activate(move |_| add_entry_cb());

    let inner = Box::builder()
        .orientation(Orientation::Vertical)
        .spacing(4)
        .margin_top(24).margin_bottom(24)
        .margin_start(24).margin_end(24)
        .build();
    inner.append(&title);
    inner.append(&hint);
    inner.append(&input_row);
    inner.append(&scroll);

    super::exam_details_card::wrap_in_card(inner)
}

pub fn append_note_row(list: &Box, text: &str, state: Rc<RefCell<AppState>>) {
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
        state.borrow_mut().exam.notes.retain(|n| n != &text_own);
        if let Some(p) = row_ref.parent() {
            if let Ok(b) = p.downcast::<Box>() { b.remove(&row_ref); }
        }
    });
}
