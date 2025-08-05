// ui.rs
use gtk::glib;
use gtk::prelude::*;
use gtk::Box as GtkBox;
use gtk::{ApplicationWindow, Entry, ListBox, ScrolledWindow, EventControllerFocus};

pub fn setup_ui(
    window: &ApplicationWindow,
    search_entry: &mut Entry,
    app_list: &mut ListBox,
) {
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
}

pub fn setup_focus_out_handler(window: &ApplicationWindow) {
    let window_weak = window.downgrade();
    
    let focus_controller = EventControllerFocus::new();
    focus_controller.connect_leave(move |_| {
        if let Some(window) = window_weak.upgrade() {
            // Add a small delay to prevent immediate closing when clicking between widgets
            glib::timeout_add_local_once(std::time::Duration::from_millis(100), move || {
                // Check if the window still exists and doesn't have focus
                if window.has_focus() {
                    return;
                }
                
                // Optionally check if any child widget has focus
                if let Some(focus_widget) = window.focus_widget() {
                    if focus_widget.has_focus() {
                        return;
                    }
                }
                
                window.close();
            });
        }
    });

    window.add_controller(focus_controller);
}
