use std::fs;
use gtk::prelude::*;
use gtk::{Application, Builder, Window, FileChooserButton};
use gdk::RGBA;
use sourceview::prelude::*;
use sourceview::{
    View,
    MarkAttributes,
    LanguageManager,
    StyleSchemeManager,
    Buffer,
};

pub struct MainWindow {
    ui: Builder,
}

impl MainWindow {
    pub fn new(application: &Application) -> Self {
        let glade_src = include_str!("MainWindow.glade");
        let main_window = Self {
            ui: Builder::new_from_string(glade_src),
        };

        main_window.connect_events();
        main_window.build_ui(application);

        main_window
    }

    fn build_ui(&self, application: &Application) {
        let window: Window = self.ui.get_object("main_window").unwrap();
        window.set_application(Some(application));

        // Set default sourceview colorscheme
        let source_buffer: Buffer = self.ui.get_object("source_buffer").unwrap();
        if let Some(theme) = StyleSchemeManager::new().get_scheme("oblivion") {
            source_buffer.set_style_scheme(Some(&theme));
        }

        // Attributes for breakpoint marks
        let source_view: View = self.ui.get_object("source_view").unwrap();
        let breakpoint_mark = MarkAttributes::new();
        breakpoint_mark.set_icon_name("media-record");
        source_view.set_mark_attributes("breakpoint", &breakpoint_mark, 1);

        // Attributes for execution marks
        let color = source_buffer.get_style_scheme().unwrap()
            .get_style("current-line").unwrap()
            .get_property_background().unwrap();
        let execution_mark = MarkAttributes::new();
        execution_mark.set_background(&color.parse::<RGBA>().unwrap());
        source_view.set_mark_attributes("execution", &execution_mark, 1);

        window.show_all();
    }

    fn connect_events(&self) {
        let file_chooser: FileChooserButton = self.ui.get_object("open_button").unwrap();
        let ui = self.ui.clone();
        file_chooser.connect_file_set(move |file_chooser| {
            if let Some(filename) = file_chooser.get_filename() {
                let file_contents = fs::read_to_string(&filename).unwrap();
                let source_buffer: Buffer = ui.get_object("source_buffer").unwrap();
                source_buffer.remove_source_marks(
                    &source_buffer.get_start_iter(),
                    &source_buffer.get_end_iter(),
                    None,
                );
                source_buffer.set_text(&file_contents);

                if let Some(lang) = LanguageManager::get_default().unwrap()
                    .guess_language(Some(filename.to_str().unwrap()), None) {
                        source_buffer.set_language(Some(&lang));
                };
            };
        });

        let source_view: View = self.ui.get_object("source_view").unwrap();
        let ui = self.ui.clone();
        source_view.connect_line_mark_activated(move |_, iter, _| {
            let source_buffer: Buffer = ui.get_object("source_buffer").unwrap();
            let mut iter = iter.clone();
            let marks = source_buffer.get_source_marks_at_iter(&mut iter, Some("breakpoint"));
            if marks.is_empty() {
                source_buffer.create_source_mark(None, "breakpoint", &iter).unwrap();
            }
            else {
                source_buffer.remove_source_marks(&iter, &iter, Some("breakpoint"));
            }
        });
    }
}
