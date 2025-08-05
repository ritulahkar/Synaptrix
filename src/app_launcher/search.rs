// Updated search.rs with debugging and name truncation
use gtk::prelude::*;
use gtk::Box as GtkBox;
use gtk::{Button, Entry, Label, ListBox, ListBoxRow};
use std::collections::HashMap;

use crate::app_info::{AppInfo, ItemType};
use crate::settings::LauncherSettings;
use crate::utils::{create_icon_widget, launch_application, open_file};

pub fn setup_search(
    search_entry: &Entry,
    app_list: &ListBox,
    apps: &HashMap<String, AppInfo>,
    recent_files: &[AppInfo],
    settings: &LauncherSettings,
) {
    // println!("DEBUG: Setting up search with {} recent files", recent_files.len());
    let app_list_clone = app_list.clone();
    let apps_clone = apps.clone();
    let recent_files_clone = recent_files.to_vec();
    let settings_clone = settings.clone();

    search_entry.connect_changed(move |entry| {
        let query = entry.text().to_lowercase();
        // println!("DEBUG: Search query: '{}'", query);
        filter_and_populate(&app_list_clone, &apps_clone, &recent_files_clone, &query, &settings_clone);
    });
}

pub fn filter_and_populate(
    list_box: &ListBox,
    apps: &HashMap<String, AppInfo>,
    recent_files: &[AppInfo],
    query: &str,
    settings: &LauncherSettings,
) {
    // println!("DEBUG: Filtering with query: '{}', {} apps, {} recent files", 
            //  query, apps.len(), recent_files.len());
    // println!("DEBUG: Recent files enabled: {}", settings.recent_files.enabled);
    
    // Clear existing items
    while let Some(child) = list_box.first_child() {
        list_box.remove(&child);
    }

    let mut all_items = Vec::new();

    // Check if it's a command (starts with common command prefixes or contains /)
    let is_command = query.starts_with('/')
        || query.starts_with("./")
        || query.contains(' ')
        || [
            "sudo", "cd", "ls", "cat", "grep", "find", "ps", "kill", "git",
        ]
        .iter()
        .any(|&cmd| query.starts_with(cmd));

    if !query.is_empty() && is_command {
        let execq = if query.starts_with('/') {
            &query[1..]
        } else if query.starts_with("./") {
            &query[2..]
        } else {
            &query
        };
        // Add command execution option
        let command_item = AppInfo {
            name: format!("💻 Run: {}", execq),
            description: "Execute command in terminal".to_string(),
            exec: if query.starts_with("sudo") {
                format!("pkexec sh -c '{}'", execq)
            } else {
                format!("sh -c '{}'", execq)
            },
            icon: Some("utilities-terminal".to_string()),
            categories: vec!["Command".to_string()],
            item_type: ItemType::Command,
            file_path: None,
        };
        all_items.push(command_item.clone());
        // println!("DEBUG: Added command item: {}", command_item.name);
    }

    // Filter applications
    let mut filtered_apps: Vec<_> = apps.values().cloned().collect();
    if !query.is_empty() {
        // let before_filter = filtered_apps.len();
        filtered_apps.retain(|app| {
            app.name.to_lowercase().contains(query)
                || app.description.to_lowercase().contains(query)
                || app
                    .categories
                    .iter()
                    .any(|cat| cat.to_lowercase().contains(query))
        });
        // println!("DEBUG: Filtered apps from {} to {}", before_filter, filtered_apps.len());
    }

    // Filter recent files
    let mut filtered_files: Vec<_> = recent_files.iter().cloned().collect();
    if !query.is_empty() {
        // let before_filter = filtered_files.len();
        filtered_files.retain(|file| {
            let matches = file.name.to_lowercase().contains(query)
                || file.description.to_lowercase().contains(query);
            if !matches {
                // println!("DEBUG: File '{}' doesn't match query '{}'", file.name, query);
            }
            matches
        });
        // println!("DEBUG: Filtered recent files from {} to {}", before_filter, filtered_files.len());
    } else {
        // println!("DEBUG: Empty query, showing all {} recent files", filtered_files.len());
    }

    // Sort applications by name
    filtered_apps.sort_by(|a, b| a.name.cmp(&b.name));

    // Add applications
    all_items.extend(filtered_apps.clone());
    // println!("DEBUG: Added {} apps to results", filtered_apps.len());

    // Add recent files
    if settings.recent_files.enabled {
        all_items.extend(filtered_files.clone());
        // println!("DEBUG: Added {} recent files to results", filtered_files.len());
    } else {
        // println!("DEBUG: Recent files disabled, not adding to results");
    }

    // Limit results
    // let before_truncate = all_items.len();
    all_items.truncate(settings.behavior.max_results);
    // println!("DEBUG: Results truncated from {} to {} (max: {})", 
            //  before_truncate, all_items.len(), settings.behavior.max_results);

    let mut last_type: Option<ItemType> = None;

    for (_i, item) in all_items.iter().enumerate() {
        // println!("DEBUG: Creating UI for item {}: {} (type: {:?})", i, item.name, item.item_type);
        
        let row = ListBoxRow::new();

        // Add CSS classes based on item type
        match item.item_type {
            ItemType::Command => row.add_css_class("command-row"),
            ItemType::RecentFile => row.add_css_class("file-row"),
            _ => row.add_css_class("app-row"),
        }

        // Add section separator class if this is the first item of a new type
        if let Some(ref last) = last_type {
            if *last != item.item_type {
                row.add_css_class("section-start");
            }
        }

        let hbox = GtkBox::new(gtk::Orientation::Horizontal, 12);
        hbox.set_margin_top(8);
        hbox.set_margin_bottom(8);
        hbox.set_margin_start(8);
        hbox.set_margin_end(8);

        // Icon with thumbnail support
        let icon_widget = create_icon_widget(&item.icon, &item.item_type, &item.file_path);
        hbox.append(&icon_widget);

        // App info
        let vbox = GtkBox::new(gtk::Orientation::Vertical, 4);
        vbox.set_hexpand(true);
        vbox.set_valign(gtk::Align::Center);

        let name_label = Label::new(Some(&item.name));
        name_label.set_halign(gtk::Align::Start);
        // Use middle ellipsize for files to preserve extensions, end for others
        let ellipsize_mode = match item.item_type {
            ItemType::RecentFile => gtk::pango::EllipsizeMode::Middle,
            _ => gtk::pango::EllipsizeMode::End,
        };
        name_label.set_ellipsize(ellipsize_mode);
        name_label.set_max_width_chars(60); // Adjust this value as needed
        name_label.add_css_class("app-name");

        vbox.append(&name_label);

        if settings.behavior.show_descriptions && !item.description.is_empty() {
            let desc_label = Label::new(Some(&item.description));
            desc_label.set_halign(gtk::Align::Start);
            desc_label.set_ellipsize(gtk::pango::EllipsizeMode::End);
            desc_label.set_max_width_chars(60);
            desc_label.add_css_class("app-description");
            vbox.append(&desc_label);
        }

        // Launch button
        let launch_btn = Button::with_label(match item.item_type {
            ItemType::Command => "Run",
            ItemType::RecentFile => "Open",
            _ => "Launch",
        });
        launch_btn.add_css_class("launch-button");
        launch_btn.set_valign(gtk::Align::Center);

        let exec_cmd = item.exec.clone();
        let file_path = item.file_path.clone();
        let item_type = item.item_type.clone();
        let window_clone = list_box
            .root()
            .and_then(|root| root.downcast::<gtk::ApplicationWindow>().ok());
        let auto_close = settings.behavior.auto_close;

        launch_btn.connect_clicked(move |_| {
            match item_type {
                ItemType::RecentFile => {
                    if let Some(ref path) = file_path {
                        // println!("DEBUG: Opening file: {:?}", path);
                        open_file(path);
                    }
                }
                _ => {
                    // println!("DEBUG: Launching application: {}", exec_cmd);
                    launch_application(&exec_cmd);
                }
            }

            if auto_close {
                if let Some(ref window) = window_clone {
                    window.close();
                }
            }
        });

        hbox.append(&vbox);
        hbox.append(&launch_btn);
        row.set_child(Some(&hbox));

        list_box.append(&row);
        last_type = Some(item.item_type.clone());
    }

    // println!("DEBUG: UI creation complete. {} items in list", all_items.len());

    // Auto-select first selectable item
    if !query.is_empty() {
        for i in 0.. {
            if let Some(row) = list_box.row_at_index(i) {
                if row.is_selectable() {
                    list_box.select_row(Some(&row));
                    break;
                }
            } else {
                break;
            }
        }
    }
}