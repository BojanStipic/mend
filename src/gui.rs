#![allow(unused)]
use std::fs;
use std::ops::Drop;
use gtk::prelude::*;
use gdk::RGBA;
use sourceview::prelude::*;

pub struct MainWindow {
    window: gtk::Window,
    header_bar: MainHeaderBar,
    toolbar: MainToolbar,
    source_window: SourceWindow,
    execution_window: ExecutionWindow,
}

impl MainWindow {
    pub fn new(parent: &gtk::Application) -> Self {
        let glade_src = include_str!("main_window.glade");
        let builder = gtk::Builder::new_from_string(glade_src);

        let window = Self {
            window: builder.get_object("main_window").unwrap(),
            header_bar: MainHeaderBar::new(&builder),
            toolbar: MainToolbar::new(&builder),
            source_window: SourceWindow::new(&builder),
            execution_window: ExecutionWindow::new(&builder),
        };
        window.connect_events();
        window.window.set_application(Some(parent));
        window.window.show_all();

        window
    }

    fn connect_events(&self) {
        let parent = self.window.clone();
        self.header_bar.primary_menu.open_about_dialog.connect_clicked(move |_| {
            AboutDialog::new(&parent);
        });

        let buffer = self.source_window.buffer.clone();
        self.header_bar.open_executable.connect_file_set(move |file_chooser| {
            if let Some(filename) = file_chooser.get_filename() {
                let file_contents = fs::read_to_string(&filename).unwrap();
                buffer.remove_source_marks(
                    &buffer.get_start_iter(),
                    &buffer.get_end_iter(),
                    None,
                );
                buffer.set_text(&file_contents);

                if let Some(lang) = sourceview::LanguageManager::get_default().unwrap()
                    .guess_language(Some(filename.to_str().unwrap()), None) {
                        buffer.set_language(Some(&lang));
                };
            };
        });

        let buffer = self.source_window.buffer.clone();
        self.source_window.view.connect_line_mark_activated(move |_, iter, _| {
            let mut iter = iter.clone();
            let marks = buffer.get_source_marks_at_iter(&mut iter, Some("breakpoint"));
            if marks.is_empty() {
                buffer.create_source_mark(None, "breakpoint", &iter).unwrap();
            }
            else {
                buffer.remove_source_marks(&iter, &iter, Some("breakpoint"));
            }
        });

        let search_bar = self.source_window.search_bar.clone();
        self.header_bar.search_toggle.connect_toggled(move |search_toggle| {
            search_bar.set_search_mode(search_toggle.get_active());
        });
        let search_toggle = self.header_bar.search_toggle.clone();
        self.source_window.search_bar.connect_property_search_mode_enabled_notify(move |search_bar| {
            search_toggle.set_active(search_bar.get_search_mode());
        });

        let buffer = self.source_window.buffer.clone();
        self.source_window.search_entry.connect_search_changed(move |search_entry| {
            let search = match search_entry.get_text() {
                Some(s) => s,
                None => return,
            };
            let settings = sourceview::SearchSettings::new();
            settings.set_search_text(Some(&search));

            let cursor = buffer.get_insert().unwrap();
            let cursor = buffer.get_iter_at_mark(&cursor);

            let context = sourceview::SearchContext::new(&buffer, Some(&settings));
            if let Some((lhs, rhs, _)) = context.forward2(&cursor) {
                buffer.select_range(&lhs, &rhs);
            };
        });

        let buffer = self.source_window.buffer.clone();
        self.source_window.search_entry.connect_next_match(move |search_entry| {
            let search = match search_entry.get_text() {
                Some(s) => s,
                None => return,
            };
            let settings = sourceview::SearchSettings::new();
            settings.set_search_text(Some(&search));

            let cursor = buffer.get_selection_bound().unwrap();
            let cursor = buffer.get_iter_at_mark(&cursor);

            let context = sourceview::SearchContext::new(&buffer, Some(&settings));
            if let Some((lhs, rhs, _)) = context.forward2(&cursor) {
                buffer.select_range(&lhs, &rhs);
            };
        });

        self.source_window.search_entry.connect_previous_match(|_| {});
        self.source_window.search_entry.connect_stop_search(|_| {});
    }
}

struct MainHeaderBar {
    headerbar: gtk::HeaderBar,
    open_executable: gtk::FileChooserButton,
    search_toggle: gtk::ToggleButton,
    open_primary_menu: gtk::Button,
    primary_menu: PrimaryMenu,
}

impl MainHeaderBar {
    fn new(builder: &gtk::Builder) -> Self {
        Self {
            headerbar: builder.get_object("main_header_bar").unwrap(),
            open_executable: builder.get_object("open_executable").unwrap(),
            search_toggle: builder.get_object("search_toggle").unwrap(),
            open_primary_menu: builder.get_object("open_primary_menu").unwrap(),
            primary_menu: PrimaryMenu::new(builder),
        }
    }
}

struct PrimaryMenu {
    popover: gtk::Popover,
    open_preferences: gtk::Button,
    open_about_dialog: gtk::Button,
}

impl PrimaryMenu {
    fn new(builder: &gtk::Builder) -> Self {
        Self {
            popover: builder.get_object("primary_popover").unwrap(),
            open_preferences: builder.get_object("open_preferences").unwrap(),
            open_about_dialog: builder.get_object("open_about_dialog").unwrap(),
        }
    }
}

struct MainToolbar {
    toolbar: gtk::Toolbar,
    run_cmd: gtk::ToolButton,
    interrupt_cmd: gtk::ToolButton,
    kill_cmd: gtk::ToolButton,
    step_cmd: gtk::ToolButton,
    next_cmd: gtk::ToolButton,
    continue_cmd: gtk::ToolButton,
}

impl MainToolbar {
    fn new(builder: &gtk::Builder) -> Self {
        Self {
            toolbar: builder.get_object("main_toolbar").unwrap(),
            run_cmd: builder.get_object("run").unwrap(),
            interrupt_cmd: builder.get_object("interrupt").unwrap(),
            kill_cmd: builder.get_object("kill").unwrap(),
            step_cmd: builder.get_object("step").unwrap(),
            next_cmd: builder.get_object("next").unwrap(),
            continue_cmd: builder.get_object("continue").unwrap(),
        }
    }
}

struct SourceWindow {
    search_bar: gtk::SearchBar,
    search_entry: gtk::SearchEntry,
    view: sourceview::View,
    buffer: sourceview::Buffer,
}

impl SourceWindow {
    fn new(builder: &gtk::Builder) -> Self {
        let window = Self {
            search_bar: builder.get_object("search_bar").unwrap(),
            search_entry: builder.get_object("search_entry").unwrap(),
            view: builder.get_object("source_view").unwrap(),
            buffer: builder.get_object("source_buffer").unwrap(),
        };

        // Set default sourceview colorscheme
        if let Some(theme) = sourceview::StyleSchemeManager::new().get_scheme("oblivion") {
            window.buffer.set_style_scheme(Some(&theme));
        }

        // Attributes for breakpoint marks
        let breakpoint_mark = sourceview::MarkAttributes::new();
        breakpoint_mark.set_icon_name("media-record");
        window.view.set_mark_attributes("breakpoint", &breakpoint_mark, 1);

        // Attributes for execution marks
        let color = window.buffer.get_style_scheme().unwrap()
            .get_style("current-line").unwrap()
            .get_property_background().unwrap();
        let execution_mark = sourceview::MarkAttributes::new();
        execution_mark.set_background(&color.parse::<RGBA>().unwrap());
        window.view.set_mark_attributes("execution", &execution_mark, 1);

        window
    }
}

struct ExecutionWindow {
    view: gtk::TextView,
}

impl ExecutionWindow {
    fn new(builder: &gtk::Builder) -> Self {
        Self {
            view: builder.get_object("execution_window").unwrap(),
        }
    }
}

struct AboutDialog {
    dialog: gtk::AboutDialog,
}

impl AboutDialog {
    fn new(parent: &gtk::Window) -> Self {
        let glade_src = include_str!("about_dialog.glade");
        let builder = gtk::Builder::new_from_string(glade_src);

        let dialog: gtk::AboutDialog = builder.get_object("about_dialog").unwrap();
        dialog.set_transient_for(Some(parent));
        dialog.run();
        Self {
            dialog
        }
    }
}

impl Drop for AboutDialog {
    fn drop(&mut self) {
        self.dialog.destroy();
    }
}
