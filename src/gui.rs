use std::fs;
use gtk::prelude::*;
use gtk::{Application, Builder, Window, FileChooserButton};
use sourceview::prelude::*;
use sourceview::{Buffer, LanguageManager, StyleSchemeManager};

pub struct MainWindow {
    ui: Builder,
}

impl MainWindow {
    pub fn new(application: &Application) -> Self {
        let glade_src = include_str!("MainWindow.glade");
        let main_window = Self {
            ui: Builder::new_from_string(glade_src),
        };

        main_window.build_ui(application);
        main_window.connect_events();

        main_window
    }

    fn build_ui(&self, application: &Application) {
        let window: Window = self.ui.get_object("main_window").unwrap();
        window.set_application(Some(application));

        let buffer: Buffer = self.ui.get_object("source_buffer").unwrap();
        StyleSchemeManager::new()
            .get_scheme("oblivion")
            .map(|theme| buffer.set_style_scheme(Some(&theme)));

        window.show_all();
    }

    fn connect_events(&self) {
        let file_chooser: FileChooserButton = self.ui.get_object("open_button").unwrap();
        let ui = self.ui.clone();
        file_chooser.connect_file_set(move |file_chooser| {
            file_chooser.get_filename().map(|filename| {
                let file_contents = fs::read_to_string(&filename).unwrap();
                let buffer: Buffer = ui.get_object("source_buffer").unwrap();
                buffer.set_text(&file_contents);

                LanguageManager::get_default().unwrap()
                    .guess_language(Some(filename.to_str().unwrap()), None)
                    .map(|lang| {
                        buffer.set_language(Some(&lang));
                    });
            });
        });
    }
}
