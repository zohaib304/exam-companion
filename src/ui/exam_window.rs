use std::cell::RefCell;
use std::rc::Rc;

use adw::prelude::*;
use adw::{Application, ApplicationWindow, HeaderBar};
use gtk::glib;
use gtk::{Box, Label, Orientation};

use crate::models::app_state::AppState;

pub fn open(app: &Application, state: Rc<RefCell<AppState>>) -> ApplicationWindow {
    let header = HeaderBar::new();
    header.set_decoration_layout(Some("icon:minimize,maximize,close"));

    // Read initial exam info from state
    let (exam_name, duration_secs, notes_text) = {
        let s = state.borrow();
        (
            s.exam.name.clone(),
            s.exam.duration_secs,
            s.exam.notes.clone(),
        )
    };

    // ─── Exam name ────────────────────────────────────────────
    let exam_label = Label::builder()
        .label(&exam_name)
        .css_classes(["title-4", "dim-label"])
        .halign(gtk::Align::Center)
        .margin_top(24)
        .build();

    let notes_box = Box::builder()
        .orientation(Orientation::Vertical)
        .spacing(4)
        .margin_top(8)
        .margin_bottom(8)
        .margin_start(24)
        .margin_end(24)
        .halign(gtk::Align::Center)
        .build();

    {
        let s = state.borrow();
        for note in &s.exam.notes {
            notes_box.append(&make_note_label(note));
        }
        notes_box.set_visible(!s.exam.notes.is_empty());
    }

    let notes_box_clone = notes_box.clone();

    // ─── Timer ────────────────────────────────────────────────
    let remaining = Rc::new(RefCell::new(duration_secs));
    let last_known = Rc::new(RefCell::new(duration_secs));

    let timer_label = Label::builder()
        .label(&format_time(*remaining.borrow()))
        .css_classes(["title-hero"])
        .halign(gtk::Align::Center)
        .valign(gtk::Align::Center)
        .vexpand(true)
        .build();

    // ─── Ticker ───────────────────────────────────────────────
    let timer_label_clone = timer_label.clone();
    let remaining_clone = remaining.clone();
    let last_known_clone = last_known.clone();
    let state_clone = state.clone();

    glib::timeout_add_seconds_local(1, move || {
        let s = state_clone.borrow();
        let current_set = s.exam.duration_secs;
        let current_notes = s.exam.notes.clone();
        drop(s);

        // ── Sync notes ────────────────────────────────────────────
        let current_notes = state_clone.borrow().exam.notes.clone();
        let rendered = notes_box_clone.observe_children().n_items() as usize;
        if current_notes.len() != rendered {
            while let Some(child) = notes_box_clone.first_child() {
                notes_box_clone.remove(&child);
            }
            for note in &current_notes {
                notes_box_clone.append(&make_note_label(note));
            }
            notes_box_clone.set_visible(!current_notes.is_empty());
        }

        // ── Check if professor changed the duration ────────────
        let last = *last_known_clone.borrow();
        if current_set != last {
            let delta = current_set as i64 - last as i64;
            let mut secs = remaining_clone.borrow_mut();
            *secs = (*secs as i64 + delta).max(0) as u32;
            *last_known_clone.borrow_mut() = current_set;
            timer_label_clone.set_text(&format_time(*secs));
            return glib::ControlFlow::Continue;
        }

        // ── Normal countdown tick ──────────────────────────────
        let mut secs = remaining_clone.borrow_mut();
        if *secs > 0 {
            *secs -= 1;
            timer_label_clone.set_text(&format_time(*secs));
            glib::ControlFlow::Continue
        } else {
            timer_label_clone.set_text("Time's Up!");
            glib::ControlFlow::Break
        }
    });

    // ─── Layout ───────────────────────────────────────────────
    let content = Box::builder().orientation(Orientation::Vertical).build();

    content.append(&header);
    content.append(&exam_label);
    content.append(&notes_box);
    content.append(&timer_label);

    // ─── Window ───────────────────────────────────────────────
    let window = ApplicationWindow::builder()
        .application(app)
        .title("Exam in Progress")
        .default_width(500)
        .default_height(400)
        .resizable(true)
        .content(&content)
        .build();

    window.present();
    window
}

fn format_time(secs: u32) -> String {
    let h = secs / 3600;
    let m = (secs % 3600) / 60;
    let s = secs % 60;
    if h > 0 {
        format!("{:02}:{:02}:{:02}", h, m, s)
    } else {
        format!("{:02}:{:02}", m, s)
    }
}


fn make_note_label(text: &str) -> Label {
    Label::builder()
        .label(&format!("• {}", text))
        .halign(gtk::Align::Center)
        .justify(gtk::Justification::Center)
        .wrap(true)
        .wrap_mode(gtk::pango::WrapMode::Word)
        .max_width_chars(60)
        .css_classes(["body", "dim-label"])
        .build()
}