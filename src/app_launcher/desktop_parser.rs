use std::collections::HashMap;
use std::fs;
use std::path::Path;

use crate::app_info::{AppInfo, ItemType};

pub fn load_applications(apps: &mut HashMap<String, AppInfo>) {
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
                        if let Ok(app_info) = parse_desktop_file(&entry.path()) {
                            apps.insert(app_info.name.clone(), app_info);
                        }
                    }
                }
            }
        }
    }
}

pub fn parse_desktop_file(path: &Path) -> Result<AppInfo, Box<dyn std::error::Error>> {
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