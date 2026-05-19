use std::cell::RefCell;
use std::rc::Rc;

use adw::prelude::*;
use adw::{Application, ApplicationWindow, HeaderBar, StatusPage};
use gtk::{Box, Button, Orientation, Stack};

use crate::models::app_state::AppState;
use crate::ui::{import, student_list, student_status_message};

pub fn build(app: &Application, state: Rc<RefCell<AppState>>) {
    let header = HeaderBar::new();
    header.set_decoration_layout(Some("icon:minimize,maximize,close"));

    let header_import_btn = Button::builder()
        .label("Import CSV")
        .css_classes(["pill"])
        .build();
    header.pack_end(&header_import_btn);

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

    let student_panel = student_list::StudentListPanel::new(state.clone());

    let stack = Stack::builder().vexpand(true).build();
    stack.add_named(&status_page, Some("empty"));
    stack.add_named(student_panel.widget(), Some("students"));
    update_stack_page(&stack, &state.borrow());

    let content = Box::builder()
        .orientation(Orientation::Vertical)
        .build();

    content.append(&header);
    content.append(&stack);

    let window = ApplicationWindow::builder()
        .application(app)
        .title("Exam Companion")
        .default_width(800)
        .default_height(600)
        .resizable(true)
        .content(&content)
        .build();

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
    connect_import(&header_import_btn);

    window.present();
}

fn update_stack_page(stack: &Stack, state: &AppState) {
    if state.students.is_empty() {
        stack.set_visible_child_name("empty");
    } else {
        stack.set_visible_child_name("students");
    }
}
