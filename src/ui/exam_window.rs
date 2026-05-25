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
    let (exam_name, duration_secs) = {
        let s = state.borrow();
        (s.exam.name.clone(), s.exam.duration_secs)
    };

    // ─── Exam name ────────────────────────────────────────────
    let exam_label = Label::builder()
        .label(&exam_name)
        .css_classes(["title-4", "dim-label"])
        .halign(gtk::Align::Center)
        .margin_top(24)
        .build();

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

        drop(s);

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
