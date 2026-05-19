use std::cell::RefCell;
use std::rc::Rc;

use adw::prelude::*;
use adw::{AlertDialog, ApplicationWindow};
use gtk::{gio, FileDialog, FileFilter};

use crate::data::import::import_student_csv;
use crate::models::app_state::AppState;

pub fn open_csv_import_dialog(
    window: &ApplicationWindow,
    state: Rc<RefCell<AppState>>,
    on_success: Rc<dyn Fn()>,
) {
    let filter = FileFilter::new();
    filter.set_name(Some("CSV files"));
    filter.add_mime_type("text/csv");
    filter.add_pattern("*.csv");

    let filters = gio::ListStore::new::<FileFilter>();
    filters.append(&filter);

    let dialog = FileDialog::builder()
        .title("Import Student List")
        .modal(true)
        .filters(&filters)
        .default_filter(&filter)
        .build();

    let window = window.clone();
    let window_for_errors = window.clone();

    dialog.open(Some(&window), None::<&gio::Cancellable>, move |result| {
        match result {
            Ok(file) => {
                let Some(path) = file.path() else {
                    show_error(
                        &window_for_errors,
                        "Import failed",
                        "Could not access the selected file.",
                    );
                    return;
                };
                match import_student_csv(&path) {
                    Ok((students, stored_path)) => {
                        state.borrow_mut().students = students;
                        eprintln!("Imported CSV saved to {}", stored_path.display());
                        on_success();
                    }
                    Err(message) => {
                        show_error(&window_for_errors, "Import failed", &message);
                    }
                }
            }
            Err(err) => {
                if err.kind::<gio::IOErrorEnum>() == Some(gio::IOErrorEnum::Cancelled) {
                    return;
                }
                show_error(&window_for_errors, "Import failed", &err.message());
            }
        }
    });
}

fn show_error(parent: &ApplicationWindow, heading: &str, body: &str) {
    let dialog = AlertDialog::new(Some(heading), Some(body));
    dialog.add_response("close", "Close");
    dialog.set_close_response("close");
    dialog.present(Some(parent));
}
