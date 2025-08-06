// ui.rs - Debug version
use gtk::prelude::*;
use gtk::Box as GtkBox;
use gtk::{ApplicationWindow, Entry, ListBox, ScrolledWindow};
use crate::settings::LauncherSettings;

pub fn setup_ui(
    window: &ApplicationWindow,
    search_entry: &mut Entry,
    app_list: &mut ListBox,
    _settings: &LauncherSettings,
) {
    // println!("Setting up UI, quit_on_close: {}", settings.behavior.quit_on_close);
    
    let vbox = GtkBox::new(gtk::Orientation::Vertical, 0);
    vbox.set_margin_top(15);
    vbox.set_margin_bottom(15);
    vbox.set_margin_start(15);
    vbox.set_margin_end(15);

    // Search entry
    *search_entry = Entry::builder()
        .placeholder_text("Search apps, run commands, or find files...")
        .build();
    search_entry.add_css_class("search-entry");

    // Scrolled window for app list
    let scrolled = ScrolledWindow::builder()
        .hscrollbar_policy(gtk::PolicyType::Never)
        .vscrollbar_policy(gtk::PolicyType::Automatic)
        .min_content_height(400)
        .build();

    *app_list = ListBox::new();
    app_list.set_selection_mode(gtk::SelectionMode::Single);
    app_list.add_css_class("app-list");
    scrolled.set_child(Some(app_list));

    vbox.append(search_entry);
    vbox.append(&scrolled);
    window.set_child(Some(&vbox));

    // Don't set up close handlers here - AppLauncher will handle them
    // println!("UI setup complete");
}