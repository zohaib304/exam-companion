use std::cell::RefCell;
use std::rc::Rc;

use adw::prelude::*;
use adw::ActionRow;
use gtk::{Box, CheckButton, Label, ListBox, Orientation, ScrolledWindow, SearchEntry};

use crate::models::app_state::AppState;

pub struct StudentListPanel {
    root: Box,
    search_entry: SearchEntry,
    list_box: ListBox,
    summary_label: Label,
    state: Rc<RefCell<AppState>>,
}

impl StudentListPanel {
    pub fn new(state: Rc<RefCell<AppState>>) -> Rc<Self> {
        let search_entry = SearchEntry::builder()
            .placeholder_text("Search by name, matriculation number, or birthdate…")
            .hexpand(true)
            .build();

        let summary_label = Label::builder()
            .halign(gtk::Align::Start)
            .css_classes(["dim-label"])
            .build();

        let list_box = ListBox::builder()
            .selection_mode(gtk::SelectionMode::None)
            .css_classes(["boxed-list"])
            .build();

        let scrolled = ScrolledWindow::builder()
            .vexpand(true)
            .child(&list_box)
            .build();

        let root = Box::builder()
            .orientation(Orientation::Vertical)
            .spacing(12)
            .margin_top(12)
            .margin_bottom(12)
            .margin_start(12)
            .margin_end(12)
            .build();

        root.append(&search_entry);
        root.append(&summary_label);
        root.append(&scrolled);

        let panel = Rc::new(Self {
            root,
            search_entry,
            list_box,
            summary_label,
            state,
        });

        let panel_for_search = panel.clone();
        panel.search_entry.connect_search_changed(move |_| {
            panel_for_search.refresh();
        });

        panel
    }

    pub fn widget(&self) -> &Box {
        &self.root
    }

    pub fn refresh(&self) {
        let query = self.search_entry.text().to_string();
        self.refresh_with_query(&query);
    }

    fn refresh_with_query(&self, query: &str) {
        while let Some(row) = self.list_box.row_at_index(0) {
            self.list_box.remove(&row);
        }

        let state = self.state.borrow();
        let indices = state.matching_student_indices(query);
        let total = state.students.len();
        let shown = indices.len();

        self.summary_label
            .set_text(&summary_text(&state, query, total, shown));

        for index in indices {
            let student = &state.students[index];
            let row = ActionRow::builder()
                .title(&student.name)
                .subtitle(format!(
                    "{} · {}",
                    student.matriculation_number, student.birthdate
                ))
                .activatable(false)
                .build();

            let present_box = Box::builder()
                .orientation(Orientation::Horizontal)
                .spacing(8)
                .valign(gtk::Align::Center)
                .build();

            present_box.append(
                &Label::builder()
                    .label("In exam")
                    .css_classes(["dim-label"])
                    .build(),
            );

            let present_check = CheckButton::builder()
                .active(student.present)
                .build();

            let state = self.state.clone();
            let summary_label = self.summary_label.clone();
            let search_entry = self.search_entry.clone();
            present_check.connect_active_notify(move |check| {
                state.borrow_mut().students[index].present = check.is_active();
                let state = state.borrow();
                let query = search_entry.text().to_string();
                let total = state.students.len();
                let shown = state.matching_student_indices(&query).len();
                summary_label.set_text(&summary_text(&state, &query, total, shown));
            });

            present_box.append(&present_check);
            row.add_suffix(&present_box);
            self.list_box.append(&row);
        }
    }
}

fn summary_text(state: &AppState, query: &str, total: usize, shown: usize) -> String {
    let present = state.students.iter().filter(|s| s.present).count();
    if query.trim().is_empty() {
        format!("{total} students · {present} present in exam")
    } else {
        format!("Showing {shown} of {total} · {present} present in exam")
    }
}
