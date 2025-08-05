// main.rs
use gtk::prelude::*;
use gtk::Application;

mod app_info;
mod app_launcher;
mod settings;
mod utils;

use app_launcher::AppLauncher;

fn main() {
    let app = Application::builder()
        .application_id("com.mint.launcher")
        .build();

    app.connect_activate(|app| {
        let launcher = AppLauncher::new(app);
        launcher.show();
    });

    app.run();
}