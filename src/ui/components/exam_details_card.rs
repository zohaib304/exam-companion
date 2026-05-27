use std::cell::RefCell;
use std::rc::Rc;
use adw::prelude::*;
use gtk::{Box, Entry, Label, Orientation};
use crate::models::app_state::AppState;

pub fn build(state: Rc<RefCell<AppState>>) -> Box {
    let title = Label::builder()
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

    {
        let s = state.borrow();
        exam_name_entry.set_text(&s.exam.name);
        professor_entry.set_text(&s.exam.professor);
    }

    let state_for_exam = state.clone();
    exam_name_entry.connect_changed(move |e| {
        state_for_exam.borrow_mut().exam.name = e.text().to_string();
    });

    let state_for_prof = state.clone();
    professor_entry.connect_changed(move |e| {
        state_for_prof.borrow_mut().exam.professor = e.text().to_string();
    });

    let inner = Box::builder()
        .orientation(Orientation::Vertical)
        .spacing(4)
        .margin_top(24).margin_bottom(24)
        .margin_start(24).margin_end(24)
        .build();

    inner.append(&title);
    inner.append(&exam_name_label);
    inner.append(&exam_name_entry);
    inner.append(&professor_label);
    inner.append(&professor_entry);

    wrap_in_card(inner)
}

pub fn wrap_in_card(inner: Box) -> Box {
    let card = Box::builder()
        .orientation(Orientation::Vertical)
        .css_classes(["card"])
        .hexpand(true)
        .vexpand(false)
        .build();
    card.append(&inner);
    card
}