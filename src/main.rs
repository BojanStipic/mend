use gtk::prelude::*;
use gio::prelude::*;
use gtk::{Window, Builder};
use std::env::args;

fn build_ui(application: &gtk::Application) {
    let glade_src = include_str!("MainWindow.glade");
    let builder = Builder::new_from_string(glade_src);

    let window: Window = builder.get_object("main_window").unwrap();
    window.set_application(Some(application));

    window.show_all();
}

fn main() {
    let application = gtk::Application::new(
        Some("bojanstipic.rdb"),
        Default::default(),
    ).unwrap();

    application.connect_activate(|app| {
        build_ui(app);
    });

    application.run(&args().collect::<Vec<_>>());
}
