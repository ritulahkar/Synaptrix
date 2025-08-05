// app_launcher.rs - Enhanced with configurable quit functionality
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
    app_ref: Option<Application>, // Store reference to quit/hide the app
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

        // Make window modal
        window.set_modal(true);
        
        let mut launcher = Self {
            apps: HashMap::new(),
            recent_files: Vec::new(),
            window,
            search_entry: Entry::new(),
            app_list: ListBox::new(),
            settings,
            app_ref: Some(app.clone()),
        };

        launcher.setup();
        launcher
    }

    fn setup(&mut self) {
        ui::setup_ui(&self.window, &mut self.search_entry, &mut self.app_list, &self.settings);
        styles::setup_styles(&self.window, &self.settings);
        
        // Setup close request handler AFTER UI is set up
        self.setup_close_handler();
        
        // Modified focus out handler to hide instead of close
        self.setup_hide_on_focus_out();
        
        desktop_parser::load_applications(&mut self.apps);
        file_loader::load_recent_files(&mut self.recent_files, &self.settings);
        keyboard::setup_keyboard_navigation(&self.window, &self.search_entry, &self.app_list);
        search::setup_search(&self.search_entry, &self.app_list, &self.apps, &self.recent_files, &self.settings);
        
        // Setup enhanced keyboard handlers
        self.setup_keyboard_handlers();
        
        self.populate_list("");
    }

    fn setup_close_handler(&self) {
        let app_weak = self.app_ref.as_ref().unwrap().downgrade();
        let quit_on_close = self.settings.behavior.quit_on_close;
        
        if quit_on_close {
            println!("Setting up NORMAL mode close handler");
            self.window.connect_close_request(move |window| {
                println!("Close request received in NORMAL mode");
                if let Some(app) = app_weak.upgrade() {
                    println!("Calling app.quit()");
                    app.quit();
                } else {
                    println!("No application found!");
                }
                gtk::glib::Propagation::Stop
            });
        } else {
            println!("Setting up DAEMON mode close handler");
            self.window.connect_close_request(|window| {
                println!("Close request received in DAEMON mode - hiding window");
                window.set_visible(false);
                gtk::glib::Propagation::Stop
            });
        }
    }

    fn setup_hide_on_focus_out(&self) {
        let window_weak = self.window.downgrade();
        let app_weak = self.app_ref.as_ref().unwrap().downgrade();
        let quit_on_close = self.settings.behavior.quit_on_close;
        let focus_controller = gtk::EventControllerFocus::new();
        
        focus_controller.connect_leave(move |_| {
            let app_weak=app_weak.clone();
            if let Some(window) = window_weak.upgrade() {
                gtk::glib::timeout_add_local_once(std::time::Duration::from_millis(100), move || {
                    if window.has_focus() {
                        return;
                    }
                    
                    if let Some(focus_widget) = window.focus_widget() {
                        if focus_widget.has_focus() {
                            return;
                        }
                    }
                    
                    // Respect the quit_on_close setting
                    if quit_on_close {
                        // println!("Focus lost in normal mode - quitting app");
                        if let Some(app) = app_weak.upgrade() {
                            app.quit();
                        }
                    } else {
                        // println!("Focus lost in daemon mode - hiding window");
                        window.set_visible(false);
                    }
                });
            }
        });
        
        self.window.add_controller(focus_controller);
    }

    fn setup_keyboard_handlers(&self) {
        let window_weak = self.window.downgrade();
        let app_weak = self.app_ref.as_ref().unwrap().downgrade();
        let quit_on_close = self.settings.behavior.quit_on_close;
        let key_controller = gtk::EventControllerKey::new();
        
        key_controller.connect_key_pressed(move |_, key, _, modifier| {
            match key {
                gtk::gdk::Key::Escape => {
                    if let Some(window) = window_weak.upgrade() {
                        if quit_on_close {
                            // Quit the entire application
                            if let Some(app) = app_weak.upgrade() {
                                println!("ESC pressed - quitting app in normal mode");
                                app.quit();
                            }
                        } else {
                            // Just hide the window (daemon mode)
                            println!("ESC pressed - hiding window in daemon mode");
                            window.set_visible(false);
                        }
                        return gtk::glib::Propagation::Stop;
                    }
                }
                gtk::gdk::Key::q | gtk::gdk::Key::Q => {
                    // Ctrl+Q to always quit the entire application regardless of settings
                    if modifier.contains(gtk::gdk::ModifierType::CONTROL_MASK) {
                        if let Some(app) = app_weak.upgrade() {
                            println!("Ctrl+Q pressed - quitting app");
                            app.quit();
                            return gtk::glib::Propagation::Stop;
                        }
                    }
                }
                _ => {}
            }
            gtk::glib::Propagation::Proceed
        });
        
        self.window.add_controller(key_controller);
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
        // Clear search when showing
        self.search_entry.set_text("");
        
        // Re-populate the list with all items
        search::filter_and_populate(
            &self.app_list,
            &self.apps,
            &self.recent_files,
            "",
            &self.settings,
        );
        
        // Show and present the window
        self.window.set_visible(true);
        self.window.present();
        self.search_entry.grab_focus();
    }

    pub fn hide(&self) {
        self.window.set_visible(false);
    }

    pub fn toggle(&self) {
        if self.window.is_visible() {
            if self.settings.behavior.quit_on_close {
                self.quit();
            } else {
                self.hide();
            }
        } else {
            self.show();
        }
    }

    pub fn quit(&self) {
        if let Some(ref app) = self.app_ref {
            println!("AppLauncher::quit() called");
            app.quit();
        }
    }

    /// Check if the app should stay in daemon mode
    pub fn is_daemon_mode(&self) -> bool {
        !self.settings.behavior.quit_on_close
    }

    // Getters remain the same
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