use gtk::gdk;
use gtk::gdk_pixbuf;
use gtk::glib;
use gtk::prelude::*;
use gtk::Box as GtkBox;
use gtk::{
    Application, ApplicationWindow, Button, Entry, Image, Label, ListBox, ListBoxRow,
    ScrolledWindow,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

#[derive(Debug, Clone, Serialize, Deserialize)]
struct LauncherSettings {
    window: WindowSettings,
    theme: ThemeSettings,
    behavior: BehaviorSettings,
    recent_files: RecentFilesSettings,
}


#[derive(Debug, Clone, Serialize, Deserialize)]
struct WindowSettings {
    width: i32,
    height: i32,
    position: String, // "center", "top", "bottom"
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ThemeSettings {
    background_color: String,
    accent_color: String,
    text_color: String,
    transparency: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct BehaviorSettings {
    max_results: usize,
    auto_close: bool,
    show_descriptions: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct RecentFilesSettings {
    enabled: bool,
    max_files: usize,
    directories: Vec<String>,
}

impl Default for LauncherSettings {
    fn default() -> Self {
        let home = std::env::var("HOME").unwrap_or_default();
        Self {
            window: WindowSettings {
                width: 700,
                height: 500,
                position: "center".to_string(),
            },
            theme: ThemeSettings {
                background_color: "rgba(248, 249, 250, 0.95)".to_string(),
                accent_color: "rgba(52, 152, 219, 0.8)".to_string(),
                text_color: "#2c3e50".to_string(),
                transparency: 0.95,
            },
            behavior: BehaviorSettings {
                max_results: 50,
                auto_close: true,
                show_descriptions: true,
            },
            recent_files: RecentFilesSettings {
                enabled: true,
                max_files: 20,
                directories: vec![
                    format!("{}/Documents", home),
                    format!("{}/Downloads", home),
                    format!("{}/Desktop", home),
                    format!("{}/Pictures", home),
                ],
            },
        }
    }
}

#[derive(Debug, Clone)]
struct AppInfo {
    name: String,
    description: String,
    exec: String,
    icon: Option<String>,
    categories: Vec<String>,
    item_type: ItemType,
    file_path: Option<PathBuf>, // Added for file handling
}

#[derive(Debug, Clone, PartialEq)]
enum ItemType {
    Application,
    Command,
    RecentFile,
}

struct AppLauncher {
    apps: HashMap<String, AppInfo>,
    recent_files: Vec<AppInfo>,
    window: ApplicationWindow,
    search_entry: Entry,
    app_list: ListBox,
    settings: LauncherSettings,
}

#[derive(Debug)]
struct XbelBookmark {
    file_path: std::path::PathBuf,
    mime_type: String,
}

impl AppLauncher {
    fn new(app: &Application) -> Self {
        let settings = Self::load_settings();

        let window = ApplicationWindow::builder()
            .application(app)
            .title("Launcher")
            .default_width(settings.window.width)
            .default_height(settings.window.height)
            .decorated(false)
            .resizable(false) // This often helps with centering
            .build();

        // let pixbuf = gdk_pixbuf::Pixbuf::from_resource("/com/example/myapp/icon.png")
        //     .expect("Failed to load icon from resource");
        // window.set_icon_name(Some(&pixbuf));

        // Make window modal and set up click-outside-to-close
        window.set_modal(true);
        window.present();

        let css_provider = gtk::CssProvider::new();
        let css_content = format!(
            r#"
            window {{
                background: {};
                border-radius: 12px;
                border: 1px solid rgba(149, 165, 166, 0.3);
                box-shadow: 0 8px 32px rgba(0, 0, 0, 0.1);
            }}
            
            .search-entry {{
                background: rgba(255, 255, 255, 0.9);
                border: 2px solid rgba(189, 195, 199, 0.4);
                border-radius: 8px;
                color: {};
                font-size: 16px;
                padding: 12px;
                margin: 10px;
            }}
            
            .search-entry:focus {{
                border-color: {};
                box-shadow: 0 0 10px rgba(52, 152, 219, 0.3);
            }}
            
            .app-list {{
                background: transparent;
                border: none;
            }}
            
            .app-row {{
                background: rgba(255, 255, 255, 0.8);
                border-radius: 8px;
                margin: 4px 8px;
                padding: 8px;
                border: 1px solid rgba(189, 195, 199, 0.4);
                transition: all 200ms ease;
            }}
            
            .app-row:hover {{
                background: rgba(236, 240, 241, 0.9);
                border-color: {};
                transform: translateY(-1px);
                box-shadow: 0 4px 12px rgba(0, 0, 0, 0.1);
            }}
            
            .app-row:selected {{
                background: {};
                border-color: rgba(52, 152, 219, 0.9);
                color: white;
            }}
            
            .app-row:selected .app-name {{
                color: white;
                font-weight: bold;
            }}
            
            .app-row:selected .app-description {{
                color: rgba(255, 255, 255, 0.9);
            }}
            
            .app-name {{
                color: {};
                font-weight: bold;
                font-size: 14px;
            }}
            
            .app-description {{
                color: rgba(127, 140, 141, 0.9);
                font-size: 11px;
            }}
            
            .launch-button {{
                background: {};
                border: none;
                border-radius: 6px;
                color: white;
                font-weight: bold;
                padding: 6px 12px;
                transition: all 200ms ease;
            }}
            
            .launch-button:hover {{
                background: rgba(41, 128, 185, 0.9);
                transform: scale(1.05);
            }}
            
            .command-row {{
                background: rgba(255, 255, 255, 0.8);
                border-radius: 8px;
                margin: 4px 8px;
                padding: 8px;
                border: 1px solid rgba(189, 195, 199, 0.4);
                transition: all 200ms ease;
                background: rgba(46, 204, 113, 0.1);
                border-color: rgba(46, 204, 113, 0.4);
            }}
            
            .command-row:selected {{
                background: rgba(46, 204, 113, 0.8);
                border-color: rgba(46, 204, 113, 1.0);
                color: white;
            }}
            
            .command-row:selected .app-name {{
                color: white;
            }}
            
            .command-row:selected .app-description {{
                color: rgba(255, 255, 255, 0.9);
            }}
            
            .file-row {{
                background: rgba(255, 255, 255, 0.8);
                border-radius: 8px;
                margin: 4px 8px;
                padding: 8px;
                border: 1px solid rgba(189, 195, 199, 0.4);
                transition: all 200ms ease;
                background: rgba(230, 126, 34, 0.1);
                border-color: rgba(230, 126, 34, 0.4);
            }}
            
            .file-row:selected {{
                background: rgba(230, 126, 34, 0.8);
                border-color: rgba(230, 126, 34, 1.0);
                color: white;
            }}
            
            .file-row:selected .app-name {{
                color: white;
            }}
            
            .file-row:selected .app-description {{
                color: rgba(255, 255, 255, 0.9);
            }}
            
            .section-separator {{
                background: rgba(189, 195, 199, 0.5);
                min-height: 5px;
                margin: 8px 16px;
            }}
            
            scrolledwindow {{
                background: transparent;
                border: none;
            }}
            .section-start {{
                border-top: 1px solid rgba(255, 255, 255, 0.1);
                margin-top: 8px;
                padding-top: 8px;
            }}


            .command-row.section-start {{
                border-top-color: rgba(100, 255, 100, 0.2);
                    }}

            .file-row.section-start {{
                border-top-color: rgba(100, 100, 255, 0.2);
                    }}
            "#,
            settings.theme.background_color,
            settings.theme.text_color,
            settings.theme.accent_color,
            settings.theme.accent_color,
            settings.theme.accent_color,
            settings.theme.text_color,
            settings.theme.accent_color,
        );

        css_provider.load_from_data(&css_content);

        if let Some(display) = gdk::Display::default() {
            gtk::style_context_add_provider_for_display(
                &display,
                &css_provider,
                gtk::STYLE_PROVIDER_PRIORITY_APPLICATION,
            );
        }

        let vbox = GtkBox::new(gtk::Orientation::Vertical, 0);
        vbox.set_margin_top(15);
        vbox.set_margin_bottom(15);
        vbox.set_margin_start(15);
        vbox.set_margin_end(15);

        // Search entry
        let search_entry = Entry::builder()
            .placeholder_text("Search apps, run commands, or find files...")
            .build();
        search_entry.add_css_class("search-entry");

        // Scrolled window for app list
        let scrolled = ScrolledWindow::builder()
            .hscrollbar_policy(gtk::PolicyType::Never)
            .vscrollbar_policy(gtk::PolicyType::Automatic)
            .min_content_height(400)
            .build();

        let app_list = ListBox::new();
        app_list.set_selection_mode(gtk::SelectionMode::Single);
        app_list.add_css_class("app-list");
        scrolled.set_child(Some(&app_list));

        vbox.append(&search_entry);
        vbox.append(&scrolled);

        window.set_child(Some(&vbox));

        let mut launcher = Self {
            apps: HashMap::new(),
            recent_files: Vec::new(),
            window,
            search_entry,
            app_list,
            settings,
        };

        launcher.setup_focus_out_handler();
        launcher.load_applications();
        launcher.load_recent_files();
        launcher.setup_keyboard_navigation();
        launcher.setup_search();
        launcher.populate_list("");

        launcher
    }

    fn load_settings() -> LauncherSettings {
        LauncherSettings::default()
    }

    fn setup_focus_out_handler(&self) {
        let window_clone = self.window.clone();

        let focus_controller = gtk::EventControllerFocus::new();
        focus_controller.connect_leave(move |_| {
            let value = window_clone.clone();
            glib::idle_add_local_once(move || {
                value.close();
            });
        });

        self.window.add_controller(focus_controller);
    }

    fn load_applications(&mut self) {
        let home_path = format!(
            "{}/.local/share/applications",
            std::env::var("HOME").unwrap_or_default()
        );
        let desktop_dirs = vec![
            "/usr/share/applications",
            "/usr/local/share/applications",
            &home_path,
        ];

        for dir in desktop_dirs {
            if let Ok(entries) = fs::read_dir(dir) {
                for entry in entries.flatten() {
                    if let Some(ext) = entry.path().extension() {
                        if ext == "desktop" {
                            if let Ok(app_info) = self.parse_desktop_file(&entry.path()) {
                                self.apps.insert(app_info.name.clone(), app_info);
                            }
                        }
                    }
                }
            }
        }
    }

    fn load_recent_files(&mut self) {
        if !self.settings.recent_files.enabled {
            return;
        }

        // Load from directories (existing functionality)
        for dir_path in &self.settings.recent_files.directories {
            let expanded_path = dir_path.replace("~", &std::env::var("HOME").unwrap_or_default());
            if let Ok(entries) = fs::read_dir(&expanded_path) {
                let mut files: Vec<_> = entries
                    .filter_map(|e| e.ok())
                    .filter(|e| e.file_type().map(|ft| ft.is_file()).unwrap_or(false))
                    .filter_map(|e| {
                        let metadata = e.metadata().ok()?;
                        let modified = metadata.modified().ok()?;
                        Some((e.path(), modified))
                    })
                    .collect();

                files.sort_by(|a, b| b.1.cmp(&a.1)); // Sort by modification time, newest first

                for (path, _) in files.into_iter().take(self.settings.recent_files.max_files) {
                    if let Some(name) = path.file_name() {
                        let app_info = AppInfo {
                            name: format!("📄 {}", name.to_string_lossy()),
                            description: format!("Recent file: {}", path.display()),
                            exec: String::new(), // Will be handled specially
                            icon: self.get_file_icon(&path),
                            categories: vec!["Recent".to_string()],
                            item_type: ItemType::RecentFile,
                            file_path: Some(path.clone()),
                        };
                        self.recent_files.push(app_info);
                    }
                }
            }
        }

        // Load from ~/.local/share/recently-used.xbel
        self.load_from_xbel();
    }

    fn load_from_xbel(&mut self) {
        let home = std::env::var("HOME").unwrap_or_default();
        let xbel_path = format!("{}/.local/share/recently-used.xbel", home);

        if let Ok(content) = fs::read_to_string(&xbel_path) {
            if let Ok(xbel_files) = self.parse_xbel(&content) {
                // Merge with existing recent files, avoiding duplicates
                let mut existing_paths: HashMap<std::path::PathBuf, usize> = HashMap::new();
                for (i, file) in self.recent_files.iter().enumerate() {
                    if let Some(path) = &file.file_path {
                        existing_paths.insert(path.clone(), i);
                    }
                }

                for xbel_file in xbel_files {
                    if let Some(ref path) = xbel_file.file_path {
                        if !existing_paths.contains_key(path) {
                            self.recent_files.push(xbel_file);
                        }
                    }
                }

                // Sort all recent files by modification time and limit count
                self.recent_files.sort_by(|a, b| {
                    let a_time = a
                        .file_path
                        .as_ref()
                        .and_then(|p| fs::metadata(p).ok())
                        .and_then(|m| m.modified().ok());
                    let b_time = b
                        .file_path
                        .as_ref()
                        .and_then(|p| fs::metadata(p).ok())
                        .and_then(|m| m.modified().ok());
                    b_time.cmp(&a_time)
                });

                self.recent_files
                    .truncate(self.settings.recent_files.max_files);
            }
        }
    }

    fn parse_xbel(&self, content: &str) -> Result<Vec<AppInfo>, Box<dyn std::error::Error>> {
        let mut files = Vec::new();
        let mut current_bookmark: Option<XbelBookmark> = None;

        // Simple XML parsing - in production, consider using a proper XML parser like quick-xml
        for line in content.lines() {
            let line = line.trim();

            if line.starts_with("<bookmark href=\"file://") {
                // Extract file path from href
                if let Some(start) = line.find("file://") {
                    if let Some(end) = line[start..].find("\"") {
                        let file_url = &line[start..start + end];
                        let file_path = file_url.replace("file://", "").replace("%20", " ");

                        current_bookmark = Some(XbelBookmark {
                            file_path: std::path::PathBuf::from(file_path),
                            mime_type: String::new(),
                        });
                    }
                }
            } else if line.contains("<mime:mime-type type=\"") {
                if let Some(bookmark) = &mut current_bookmark {
                    if let Some(start) = line.find("type=\"") {
                        if let Some(end) = line[start + 6..].find("\"") {
                            bookmark.mime_type = line[start + 6..start + 6 + end].to_string();
                        }
                    }
                }
            } else if line.starts_with("</bookmark>") {
                if let Some(bookmark) = current_bookmark.take() {
                    // Only include files that actually exist
                    if bookmark.file_path.exists() {
                        if let Some(name) = bookmark.file_path.file_name() {
                            let app_info = AppInfo {
                                name: format!("📄 {}", name.to_string_lossy()),
                                description: format!(
                                    "Recent file: {}",
                                    bookmark.file_path.display()
                                ),
                                exec: String::new(),
                                icon: self.get_file_icon(&bookmark.file_path),
                                categories: vec!["Recent".to_string()],
                                item_type: ItemType::RecentFile,
                                file_path: Some(bookmark.file_path),
                            };
                            files.push(app_info);
                        }
                    }
                }
            }
        }

        Ok(files)
    }

    fn get_file_icon(&self, path: &Path) -> Option<String> {
        match path.extension()?.to_str()? {
            "pdf" => Some("application-pdf".to_string()),
            "doc" | "docx" => Some("application-msword".to_string()),
            "xls" | "xlsx" => Some("application-vnd.ms-excel".to_string()),
            "ppt" | "pptx" => Some("application-vnd.ms-powerpoint".to_string()),
            "txt" => Some("text-x-generic".to_string()),
            "png" | "jpg" | "jpeg" | "gif" | "bmp" | "tiff" => Some("image-x-generic".to_string()),
            "mp3" | "wav" | "ogg" | "flac" => Some("audio-x-generic".to_string()),
            "mp4" | "avi" | "mkv" | "mov" | "webm" => Some("video-x-generic".to_string()),
            "zip" | "tar" | "gz" | "rar" | "7z" => Some("package-x-generic".to_string()),
            _ => Some("text-x-generic".to_string()),
        }
    }

    fn parse_desktop_file(&self, path: &Path) -> Result<AppInfo, Box<dyn std::error::Error>> {
        let content = fs::read_to_string(path)?;
        let mut name = String::new();
        let mut description = String::new();
        let mut exec = String::new();
        let mut icon = None;
        let mut categories = Vec::new();
        let mut no_display = false;
        let mut hidden = false;

        for line in content.lines() {
            let line = line.trim();
            if line.starts_with("Name=") && name.is_empty() {
                name = line[5..].to_string();
            } else if line.starts_with("Comment=") && description.is_empty() {
                description = line[8..].to_string();
            } else if line.starts_with("Exec=") {
                exec = line[5..].to_string();
            } else if line.starts_with("Icon=") {
                if icon.is_none() {
                    icon = Some(line[5..].to_string());
                }
            } else if line.starts_with("Categories=") {
                categories = line[11..]
                    .split(';')
                    .filter(|s| !s.is_empty())
                    .map(|s| s.to_string())
                    .collect();
            } else if line == "NoDisplay=true" {
                no_display = true;
            } else if line == "Hidden=true" {
                hidden = true;
            }
        }

        if name.is_empty() || exec.is_empty() || no_display || hidden {
            return Err("Invalid or hidden desktop entry".into());
        }

        // Clean up exec command
        exec = exec
            .split_whitespace()
            .filter(|s| !s.starts_with('%'))
            .collect::<Vec<_>>()
            .join(" ");

        Ok(AppInfo {
            name,
            description,
            exec,
            icon,
            categories,
            item_type: ItemType::Application,
            file_path: None,
        })
    }

    fn setup_keyboard_navigation(&self) {
        let app_list = self.app_list.clone();
        let search_entry = self.search_entry.clone();
        let window = self.window.clone();

        let key_controller = gtk::EventControllerKey::new();

        key_controller.connect_key_pressed(move |_, key, _, _| match key {
            gdk::Key::Down => {
                if let Some(selected) = app_list.selected_row() {
                    if let Some(next) = selected.next_sibling() {
                        if let Some(next_row) = next.downcast_ref::<ListBoxRow>() {
                            app_list.select_row(Some(next_row));
                            next_row.grab_focus();
                        }
                    }
                } else if let Some(first) = app_list.row_at_index(0) {
                    app_list.select_row(Some(&first));
                    first.grab_focus();
                }
                glib::Propagation::Stop
            }
            gdk::Key::Up => {
                if let Some(selected) = app_list.selected_row() {
                    if let Some(prev) = selected.prev_sibling() {
                        if let Some(prev_row) = prev.downcast_ref::<ListBoxRow>() {
                            app_list.select_row(Some(prev_row));
                            prev_row.grab_focus();
                        }
                    }
                }
                glib::Propagation::Stop
            }
            gdk::Key::Return => {
                if let Some(selected) = app_list.selected_row() {
                    Self::activate_selected_row(&selected);
                }
                glib::Propagation::Stop
            }
            gdk::Key::Escape => {
                window.close();
                glib::Propagation::Stop
            }
            _ => {
                if !search_entry.has_focus() {
                    search_entry.grab_focus();
                }
                glib::Propagation::Proceed
            }
        });

        self.window.add_controller(key_controller);

        let app_list_clone = self.app_list.clone();
        self.search_entry.connect_activate(move |_| {
            if let Some(first_row) = app_list_clone.row_at_index(0) {
                Self::activate_selected_row(&first_row);
            }
        });
    }

    fn activate_selected_row(row: &ListBoxRow) {
        if let Some(child) = row.child() {
            Self::find_and_click_button(&child);
        }
    }

    fn find_and_click_button(widget: &gtk::Widget) {
        if let Some(button) = widget.downcast_ref::<Button>() {
            button.emit_clicked();
            return;
        }

        if let Some(container) = widget.downcast_ref::<GtkBox>() {
            let mut child = container.first_child();
            while let Some(current_child) = child {
                Self::find_and_click_button(&current_child);
                child = current_child.next_sibling();
            }
        }
    }

    fn setup_search(&mut self) {
        let app_list = self.app_list.clone();
        let apps = self.apps.clone();
        let recent_files = self.recent_files.clone();
        let settings = self.settings.clone();

        self.search_entry.connect_changed(move |entry| {
            let query = entry.text().to_lowercase();
            Self::filter_and_populate(&app_list, &apps, &recent_files, &query, &settings);
        });
    }

    fn populate_list(&mut self, query: &str) {
        Self::filter_and_populate(
            &self.app_list,
            &self.apps,
            &self.recent_files,
            query,
            &self.settings,
        );
    }

    fn filter_and_populate(
        list_box: &ListBox,
        apps: &HashMap<String, AppInfo>,
        recent_files: &[AppInfo],
        query: &str,
        settings: &LauncherSettings,
    ) {
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
            all_items.push(command_item);
        }

        // Filter applications
        let mut filtered_apps: Vec<_> = apps.values().cloned().collect();
        if !query.is_empty() {
            filtered_apps.retain(|app| {
                app.name.to_lowercase().contains(query)
                    || app.description.to_lowercase().contains(query)
                    || app
                        .categories
                        .iter()
                        .any(|cat| cat.to_lowercase().contains(query))
            });
        }

        // Filter recent files
        let mut filtered_files: Vec<_> = recent_files.iter().cloned().collect();
        if !query.is_empty() {
            filtered_files.retain(|file| {
                file.name.to_lowercase().contains(query)
                    || file.description.to_lowercase().contains(query)
            });
        }

        // Sort applications by name
        filtered_apps.sort_by(|a, b| a.name.cmp(&b.name));

        // Add applications
        all_items.extend(filtered_apps);

        // Add recent files
        if settings.recent_files.enabled {
            all_items.extend(filtered_files);
        }

        // Limit results
        all_items.truncate(settings.behavior.max_results);

        let mut last_type: Option<ItemType> = None;

        for item in &all_items {
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
            let icon_widget =
                Self::create_icon_widget(&item.icon, &item.item_type, &item.file_path);
            hbox.append(&icon_widget);

            // App info
            let vbox = GtkBox::new(gtk::Orientation::Vertical, 4);
            vbox.set_hexpand(true);
            vbox.set_valign(gtk::Align::Center);

            let name_label = Label::new(Some(&item.name));
            name_label.set_halign(gtk::Align::Start);
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
                .and_then(|root| root.downcast::<ApplicationWindow>().ok());
            let auto_close = settings.behavior.auto_close;

            launch_btn.connect_clicked(move |_| {
                match item_type {
                    ItemType::RecentFile => {
                        if let Some(ref path) = file_path {
                            Self::open_file(path);
                        }
                    }
                    _ => {
                        Self::launch_application(&exec_cmd);
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

    fn create_icon_widget(
        icon_name: &Option<String>,
        item_type: &ItemType,
        file_path: &Option<PathBuf>,
    ) -> Image {
        let icon = Image::new();
        icon.set_pixel_size(48);
        icon.set_valign(gtk::Align::Center);

        // For image files, try to create thumbnails
        if let Some(path) = file_path {
            if Self::is_image_file(path) {
                if let Ok(thumbnail) = Self::create_thumbnail(path, 48) {
                    icon.set_from_file(Some(&thumbnail));
                    return icon;
                }
            }
        }

        if let Some(icon_name) = icon_name {
            // Try icon theme first (for system menu icons)
            let theme = gtk::IconTheme::for_display(&gdk::Display::default().unwrap());
            if theme.has_icon(icon_name) {
                icon.set_icon_name(Some(icon_name));
                return icon;
            }

            let icon_paths = vec![
                // Try exact name first
                icon_name.clone(),
                // Try with various sizes in hicolor theme
                format!("/usr/share/icons/hicolor/48x48/apps/{}.png", icon_name),
                format!("/usr/share/icons/hicolor/32x32/apps/{}.png", icon_name),
                format!("/usr/share/icons/hicolor/24x24/apps/{}.png", icon_name),
                format!("/usr/share/icons/hicolor/scalable/apps/{}.svg", icon_name),
                // Try in different icon themes
                format!("/usr/share/icons/Mint-X/apps/48/{}.png", icon_name),
                format!("/usr/share/icons/Mint-Y/apps/48/{}.png", icon_name),
                format!("/usr/share/icons/breeze/apps/48/{}.svg", icon_name),
                format!("/usr/share/icons/oxygen/48x48/apps/{}.png", icon_name),
                // Try pixmaps
                format!("/usr/share/pixmaps/{}.png", icon_name),
                format!("/usr/share/pixmaps/{}.svg", icon_name),
                format!("/usr/share/pixmaps/{}.xpm", icon_name),
                // Try flatpak icons
                format!("/var/lib/flatpak/app/{}/current/active/export/share/icons/hicolor/48x48/apps/{}.png", icon_name, icon_name),
                // Try application-specific directories
                format!("/usr/share/{}/icons/{}.png", icon_name, icon_name),
                format!("/opt/{}/share/icons/{}.png", icon_name, icon_name),
            ];

            for path in icon_paths {
                if Path::new(&path).exists() {
                    icon.set_from_file(Some(&path));
                    return icon;
                }
            }
        }

        // Fallback icons based on item type
        let fallback_icon = match item_type {
            ItemType::Application => "application-x-executable",
            ItemType::Command => "utilities-terminal",
            ItemType::RecentFile => "text-x-generic",
        };

        icon.set_icon_name(Some(fallback_icon));
        icon
    }

    fn is_image_file(path: &Path) -> bool {
        if let Some(ext) = path.extension() {
            matches!(
                ext.to_str().unwrap_or("").to_lowercase().as_str(),
                "png" | "jpg" | "jpeg" | "gif" | "bmp" | "tiff" | "webp" | "svg"
            )
        } else {
            false
        }
    }

    fn create_thumbnail(path: &Path, size: i32) -> Result<PathBuf, Box<dyn std::error::Error>> {
        let cache_dir = format!(
            "{}/.cache/launcher-thumbnails",
            std::env::var("HOME").unwrap_or_default()
        );
        fs::create_dir_all(&cache_dir)?;

        let filename = format!("{:x}", md5::compute(path.to_string_lossy().as_bytes()));
        let thumbnail_path = PathBuf::from(&cache_dir).join(format!("{}.png", filename));

        if !thumbnail_path.exists() {
            // Use ImageMagick to create thumbnail
            Command::new("convert")
                .arg(path)
                .arg("-thumbnail")
                .arg(format!("{}x{}", size, size))
                .arg(&thumbnail_path)
                .output()?;
        }

        Ok(thumbnail_path)
    }

    fn launch_application(exec: &str) {
        let mut cmd = if exec.contains("pkexec") {
            let parts: Vec<&str> = exec.split_whitespace().collect();
            let mut command = Command::new("pkexec");
            for part in parts.iter().skip(1) {
                command.arg(part);
            }
            command
        } else {
            let parts: Vec<&str> = exec.split_whitespace().collect();
            if let Some(program) = parts.first() {
                let mut command = Command::new(program);
                for arg in parts.iter().skip(1) {
                    command.arg(arg);
                }
                command
            } else {
                return;
            }
        };

        // Detach from parent process
        cmd.spawn().unwrap_or_else(|e| {
            eprintln!("Failed to launch application: {}", e);
            std::process::exit(1);
        });
    }

    fn open_file(path: &Path) {
        // Use xdg-open to open files with default applications
        Command::new("xdg-open")
            .arg(path)
            .spawn()
            .unwrap_or_else(|e| {
                eprintln!("Failed to open file: {}", e);
                std::process::exit(1);
            });
    }

    fn show(&self) {
        self.window.present();
        self.search_entry.grab_focus();
    }
}

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
