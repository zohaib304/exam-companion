mod data;
mod models;
mod ui;

use std::cell::RefCell;
use std::rc::Rc;

use adw::prelude::*;
use adw::Application;

use models::app_state::AppState;

const APP_ID: &str = "com.exa
mple.ExamCompanion";

fn main() {
    let state = Rc::new(RefCell::new(AppState::default()));

    let app = Application::builder()
        .application_id(APP_ID)
        .build();

    let state = state.clone();
    app.connect_activate(move |app| ui::home::build(app, state.clone()));
    app.run();
}
