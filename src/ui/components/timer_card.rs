use std::cell::RefCell;
use std::rc::Rc;
use adw::prelude::*;
use gtk::{Box, Button, Label, Orientation};
use crate::models::app_state::AppState;
use crate::models::exam_event::EventKind;

// Returns the shared duration_mins so home.rs can read it on Start
pub fn duration_mins(state: Rc<RefCell<AppState>>) -> Rc<RefCell<u32>> {
    Rc::new(RefCell::new(state.borrow().exam.duration_secs / 60))
}

pub fn build(state: Rc<RefCell<AppState>>) -> Box {
    let mins = Rc::new(RefCell::new(state.borrow().exam.duration_secs / 60));

    let duration_label = Label::builder()
        .label(&format!("{} min", mins.borrow()))
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

    // Minus
    let mins_m = mins.clone();
    let lbl_m  = duration_label.clone();
    let btn_m  = minus_btn.clone();
    let st_m   = state.clone();
    minus_btn.connect_clicked(move |_| {
        let mut m = mins_m.borrow_mut();
        if *m > 90 { *m -= 5; }
        lbl_m.set_text(&format!("{} min", *m));
        let mut s = st_m.borrow_mut();
        let old_secs = s.exam.duration_secs;
        s.exam.duration_secs = *m * 60;
        // Only log as a time extension event when the exam is running
        if s.timer_running {
            let added = s.exam.duration_secs as i64 - old_secs as i64;
            let new_total = s.exam.duration_secs;
            s.log_event(EventKind::TimeExtended {
                added_secs: added,
                new_total_secs: new_total,
            });
        }
        btn_m.set_sensitive(*m > 90);
    });

    // Plus
    let mins_p = mins.clone();
    let lbl_p  = duration_label.clone();
    let btn_p  = minus_btn.clone();
    let st_p   = state.clone();
    plus_btn.connect_clicked(move |_| {
        let mut m = mins_p.borrow_mut();
        *m += 5;
        lbl_p.set_text(&format!("{} min", *m));
        let mut s = st_p.borrow_mut();
        let old_secs = s.exam.duration_secs;
        s.exam.duration_secs = *m * 60;
        // Only log as a time extension event when the exam is running
        if s.timer_running {
            let added = s.exam.duration_secs as i64 - old_secs as i64;
            let new_total = s.exam.duration_secs;
            s.log_event(EventKind::TimeExtended {
                added_secs: added,
                new_total_secs: new_total,
            });
        }
        btn_p.set_sensitive(true);
    });

    let controls = Box::builder()
        .orientation(Orientation::Horizontal)
        .spacing(12)
        .halign(gtk::Align::Center)
        .margin_top(8)
        .build();
    controls.append(&minus_btn);
    controls.append(&duration_label);
    controls.append(&plus_btn);

    let step = Label::builder()
        .label("Step: 5 min  |  Minimum: 90 min")
        .halign(gtk::Align::Center)
        .css_classes(["caption", "dim-label"])
        .margin_top(4)
        .build();

    let title = Label::builder()
        .label("Exam Duration")
        .halign(gtk::Align::Start)
        .css_classes(["title-4"])
        .margin_bottom(12)
        .build();

    let inner = Box::builder()
        .orientation(Orientation::Vertical)
        .spacing(4)
        .margin_top(24).margin_bottom(24)
        .margin_start(24).margin_end(24)
        .build();
    inner.append(&title);
    inner.append(&controls);
    inner.append(&step);

    super::exam_details_card::wrap_in_card(inner)
}
