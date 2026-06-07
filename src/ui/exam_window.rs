use std::cell::RefCell;
use std::rc::Rc;

use adw::prelude::*;
use adw::{Application, ApplicationWindow, HeaderBar};
use gtk::glib;
use gtk::{Box, Label, Orientation, Separator};

use crate::models::app_state::AppState;
use crate::models::exam_event::EventKind;
use gtk::DrawingArea;
use cairo::Context;
use std::f64::consts::PI;

pub fn open(app: &Application, state: Rc<RefCell<AppState>>) -> ApplicationWindow {
    let header = HeaderBar::new();
    header.set_decoration_layout(Some("icon:minimize,maximize,close"));

    // Read initial exam info from state
    let (exam_name, professor_name, duration_secs) = {
        let s = state.borrow();
        (
            s.exam.name.clone(),
            s.exam.professor.clone(),
            s.exam.duration_secs,
        )
    };

    // ─── Exam name ────────────────────────────────────────────
    let course_title = Label::builder()
        .label("Course")
        .css_classes(["heading"])
        .halign(gtk::Align::Start)
        .margin_start(40)
        .margin_top(20)
        .build();

    let course_value = Label::builder()
        .label(&exam_name)
        .css_classes(["title-3"])
        .halign(gtk::Align::Start)
        .margin_start(40)
        .build();

    let professor_title = Label::builder()
        .label("Professor")
        .css_classes(["heading"])
        .halign(gtk::Align::Start)
        .margin_start(40)
        .margin_top(15)
        .build();

    let professor_value = Label::builder()
        .label(&professor_name)
        .halign(gtk::Align::Start)
        .margin_start(40)
        .build();

    let notes_title = Label::builder()
        .label("Notes")
        .css_classes(["heading"])
        .halign(gtk::Align::Start)
        .margin_start(40)
        .margin_top(15)
        .build();

    let notes_box = Box::builder()
        .orientation(Orientation::Vertical)
        .spacing(4)
        .margin_top(8)
        .margin_bottom(8)
        .margin_start(24)
        .margin_end(24)
        .halign(gtk::Align::Start)
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
    let total_duration = duration_secs;

    let timer_title = Label::builder()
    .label("Time Remaining")
    .css_classes(["heading"])
    .halign(gtk::Align::Center)
    .build();

    let timer_label = Label::builder()
        .label(&format_time(*remaining.borrow()))
        .css_classes(["title-1"])
        .halign(gtk::Align::Center)
        .valign(gtk::Align::Center)
        .vexpand(false)
        .margin_top(30)
        .margin_bottom(30)
        .build();

        // ─── Progress Ring ────────────────────────────────────────
    let progress_ring = DrawingArea::builder()
        .width_request(150)
        .height_request(150)
        .build();

    let remaining_for_ring = remaining.clone();
    progress_ring.set_draw_func(move |_, cr: &Context, width, height| {
        let secs = *remaining_for_ring.borrow();
        let progress = secs as f64 / total_duration.max(1) as f64;
        let center_x = width as f64 / 2.0;
        let center_y = height as f64 / 2.0;
        let radius = 55.0;
        cr.set_source_rgb(0.88, 0.92, 1.0);
        cr.set_line_width(12.0);
        cr.arc(center_x, center_y, radius, 0.0, 2.0 * PI);
        cr.stroke().unwrap();
        cr.set_source_rgb(0.13, 0.39, 0.96);
        cr.set_line_width(12.0);
        cr.arc(center_x, center_y, radius, -PI / 2.0, (-PI / 2.0) + (2.0 * PI * progress));
        cr.stroke().unwrap();
        cr.set_source_rgb(0.13, 0.39, 0.96);
        cr.arc(center_x, center_y, 6.0, 0.0, 2.0 * PI);
        cr.fill().unwrap();
    });

    // ─── Ticker ───────────────────────────────────────────────
    let timer_label_clone = timer_label.clone();
    let remaining_clone = remaining.clone();
    let progress_ring_clone = progress_ring.clone();
    let last_known_clone = last_known.clone();
    let state_clone = state.clone();

    glib::timeout_add_seconds_local(1, move || {
        let s = state_clone.borrow();
        let current_set = s.exam.duration_secs;
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
            progress_ring_clone.queue_draw();

            glib::ControlFlow::Continue
        } else {
            timer_label_clone.set_text("Time's Up!");
            // Log ExamEnded when timer naturally runs out
            let mut s = state_clone.borrow_mut();
            if !s.exam_ended {
                s.exam_ended = true;
                s.log_event(EventKind::ExamEnded);
            }
            glib::ControlFlow::Break
        }
    });

    // ─── Layout ───────────────────────────────────────────────
    let content = Box::builder().orientation(Orientation::Vertical).build();

    content.append(&header);

    let left_panel = Box::builder()
        .orientation(Orientation::Vertical)
        .spacing(6)
        .margin_top(20)
        .margin_start(20)
        .build();

    left_panel.append(&course_title);
    left_panel.append(&course_value);

    left_panel.append(&professor_title);
    left_panel.append(&professor_value);

    let notes_panel = Box::builder()
        .orientation(Orientation::Vertical)
        .spacing(10)
        .margin_start(30)
        .margin_end(30)
        .width_request(250)
        .build();

    notes_panel.append(&notes_title);
    notes_panel.append(&notes_box);

    let timer_panel = gtk::Frame::builder()
        .hexpand(true)
        .margin_top(30)
        .margin_bottom(30)
        .margin_end(30)
        .width_request(380)
        .height_request(260)
        .build();

    let timer_content = Box::builder()
        .orientation(Orientation::Horizontal)
        .spacing(30)
        .margin_top(30)
        .margin_bottom(30)
        .margin_start(30)
        .margin_end(30)
        .halign(gtk::Align::Center)
        .valign(gtk::Align::Center)
        .build();

        let ring_box = Box::builder()
            .orientation(Orientation::Vertical)
            .spacing(10)
            .halign(gtk::Align::Center)
            .valign(gtk::Align::Center)
            .build();

        ring_box.append(&progress_ring);

    let timer_text_box = Box::builder()
        .orientation(Orientation::Vertical)
        .spacing(10)
        .halign(gtk::Align::Center)
        .valign(gtk::Align::Center)
        .build();

    timer_text_box.append(&timer_title);
    timer_text_box.append(&timer_label);

    timer_content.append(&ring_box);
    timer_content.append(&timer_text_box);

    timer_panel.set_child(Some(&timer_content));

    let separator = Separator::new(Orientation::Vertical);

    let body = Box::builder()
        .orientation(Orientation::Horizontal)
        .spacing(20)
        .margin_top(20)
        .margin_bottom(20)
        .margin_start(20)
        .margin_end(20)
        .hexpand(true)
        .build();

    body.append(&left_panel);

    body.append(
        &Separator::new(Orientation::Vertical)
    );

    body.append(&notes_panel);

    body.append(
        &Separator::new(Orientation::Vertical)
    );

    body.append(&timer_panel);

    content.append(&body);
    // ─── Window ───────────────────────────────────────────────
    let window = ApplicationWindow::builder()
        .application(app)
        .title("Exam in Progress")
        .default_width(1000)
        .default_height(350)
        .resizable(true)
        .content(&content)
        .build();

    // Log ExamEnded when the exam window is closed manually
    let state_for_close = state.clone();
    window.connect_close_request(move |_| {
        let mut s = state_for_close.borrow_mut();
        if !s.exam_ended {
            s.exam_ended = true;
            s.log_event(EventKind::ExamEnded);
        }
        glib::Propagation::Proceed
    });

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
