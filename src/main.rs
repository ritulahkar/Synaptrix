// main.rs
use gtk::prelude::*;
use gtk::{Application, gio};
use std::rc::Rc;
use std::cell::RefCell;

mod app_info;
mod app_launcher;
mod settings;
mod utils;

use app_launcher::AppLauncher;
use settings::LauncherSettings;

fn main() {
    // Load settings first to determine the application behavior
    let settings = LauncherSettings::load();
    
    let app_flags = if settings.behavior.quit_on_close {
        // Normal mode - allow multiple instances
        gio::ApplicationFlags::HANDLES_COMMAND_LINE | gio::ApplicationFlags::NON_UNIQUE
    } else {
        // Daemon mode - single instance only
        gio::ApplicationFlags::HANDLES_COMMAND_LINE
    };

    let app = Application::builder()
        .application_id("com.github.ritulahkar.synaptrix")
        .flags(app_flags)
        .build();

    // Store launcher in Rc<RefCell<>> to share between closures
    let launcher: Rc<RefCell<Option<AppLauncher>>> = Rc::new(RefCell::new(None));
    let launcher_clone = launcher.clone();

    // Primary instance activation
    app.connect_activate(move |app| {
        if let Ok(mut launcher_ref) = launcher_clone.try_borrow_mut() {
            if launcher_ref.is_none() {
                // First time - create the launcher
                *launcher_ref = Some(AppLauncher::new(app));
            }
            // Show the launcher window
            if let Some(ref launcher) = *launcher_ref {
                launcher.show();
            }
        }
    });

    // Handle command line for both primary and remote instances
    let launcher_clone2 = launcher.clone();
    app.connect_command_line(move |app, _cmdline| {
        // Check if we're the primary instance
        if app.is_remote() {
            // We're a remote instance trying to communicate with primary
            return 0;
        }

        // We're the primary instance, handle the command
        if let Ok(launcher_ref) = launcher_clone2.try_borrow() {
            if let Some(ref launcher) = *launcher_ref {
                // Show the existing window
                launcher.show();
                return 0;
            }
        }

        // Fallback: create launcher if it doesn't exist
        app.activate();
        0
    });

    // Handle startup
    app.connect_startup(move |_app| {
        if !settings.behavior.quit_on_close {
            println!("Starting in daemon mode - will stay in memory when closed");
        } else {
            println!("Starting in normal mode - will quit when closed");
        }
    });

    // No need for window_removed handler - we handle quitting directly in close_request

    app.run();
}