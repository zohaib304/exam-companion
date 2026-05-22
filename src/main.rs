use adw::prelude::*;
use adw::{Application, ApplicationWindow, HeaderBar};
use gtk::{Box, Label, Orientation};

const APP_ID: &str = "com.example.ExamCompanion";

fn main() {
    let app = Application::builder()
        .application_id(APP_ID)
        .build();

    app.connect_activate(build_ui);
    app.run();
}

fn build_ui(app: &Application) {
    // Header bar
    let header = HeaderBar::new();
    header.set_decoration_layout(Some("icon:minimize,maximize,close"));

    // Exam Companion label
    let label = Label::builder()
        .label("Exam Companion!")
        .css_classes(["title-1"])
        .vexpand(true)
        .hexpand(true)
        .halign(gtk::Align::Center)
        .valign(gtk::Align::Center)
        .build();

    // Main vertical layout
    let content = Box::builder()
        .orientation(Orientation::Vertical)
        .build();

    content.append(&header);
    content.append(&label);

    // Window
    let window = ApplicationWindow::builder()
        .application(app)
        .title("Exam Companion")
        .default_width(800)
        .default_height(600)
        .resizable(true)
        .content(&content)
        .build();

    window.present();
}