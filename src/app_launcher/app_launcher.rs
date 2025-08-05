//  app_launcher.rs
use gtk::prelude::*;
use gtk::{Application, ApplicationWindow, Entry, ListBox};
use std::collections::HashMap;

use crate::app_info::AppInfo;
use crate::settings::LauncherSettings;
use super::{ui, keyboard, search, file_loader, desktop_parser, styles};

pub struct AppLauncher {
    apps: HashMap<String, AppInfo>,
    recent_files: Vec<AppInfo>,
    window: ApplicationWindow,
    search_entry: Entry,
    app_list: ListBox,
    settings: LauncherSettings,
}

impl AppLauncher {
    pub fn new(app: &Application) -> Self {
        let settings = LauncherSettings::load();

        let window = ApplicationWindow::builder()
            .application(app)
            .title("Launcher")
            .default_width(settings.window.width)
            .default_height(settings.window.height)
            .decorated(false)
            .resizable(false)
            .build();

        // Make window modal and set up click-outside-to-close
        window.set_modal(true);
        window.present();

        let mut launcher = Self {
            apps: HashMap::new(),
            recent_files: Vec::new(),
            window,
            search_entry: Entry::new(),
            app_list: ListBox::new(),
            settings,
        };

        launcher.setup();
        launcher
    }

    fn setup(&mut self) {
        ui::setup_ui(&self.window, &mut self.search_entry, &mut self.app_list);
        styles::setup_styles(&self.window, &self.settings);
        ui::setup_focus_out_handler(&self.window);
        desktop_parser::load_applications(&mut self.apps);
        file_loader::load_recent_files(&mut self.recent_files, &self.settings);
        keyboard::setup_keyboard_navigation(&self.window, &self.search_entry, &self.app_list);
        search::setup_search(&self.search_entry, &self.app_list, &self.apps, &self.recent_files, &self.settings);
        self.populate_list("");
    }

    fn populate_list(&mut self, query: &str) {
        search::filter_and_populate(
            &self.app_list,
            &self.apps,
            &self.recent_files,
            query,
            &self.settings,
        );
    }

    pub fn show(&self) {
        self.window.present();
        self.search_entry.grab_focus();
    }

    pub fn window(&self) -> &ApplicationWindow {
        &self.window
    }

    pub fn search_entry(&self) -> &Entry {
        &self.search_entry
    }

    pub fn app_list(&self) -> &ListBox {
        &self.app_list
    }

    pub fn settings(&self) -> &LauncherSettings {
        &self.settings
    }
}