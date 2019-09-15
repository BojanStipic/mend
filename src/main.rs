mod gui;

use gio::prelude::*;
use gtk::Application;
use std::env::args;

use gui::MainWindow;

fn main() {
    let application = Application::new(
        Some("bojanstipic.mend"),
        Default::default(),
    ).unwrap();

    application.connect_activate(|app| {
        MainWindow::new(app);
    });

    application.run(&args().collect::<Vec<_>>());
}
