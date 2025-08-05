// file_loader.rs
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

use crate::app_info::{AppInfo, ItemType, XbelBookmark};
use crate::settings::LauncherSettings;
use crate::utils::get_file_icon;

pub fn load_recent_files(recent_files: &mut Vec<AppInfo>, settings: &LauncherSettings) {
    if !settings.recent_files.enabled {
        // println!("DEBUG: Recent files disabled in settings");
        return;
    }

    // println!("DEBUG: Loading recent files...");
    // println!("DEBUG: Max files setting: {}", settings.recent_files.max_files);
    // println!("DEBUG: Directories to scan: {:?}", settings.recent_files.directories);

    // Load from directories (existing functionality)
    let mut directory_files = Vec::new();
    for dir_path in &settings.recent_files.directories {
        let expanded_path = dir_path.replace("~", &std::env::var("HOME").unwrap_or_default());
        // println!("DEBUG: Scanning directory: {}", expanded_path);
        
        match fs::read_dir(&expanded_path) {
            Ok(entries) => {
                let mut files: Vec<_> = entries
                    .filter_map(|e| e.ok())
                    .filter(|e| {
                        let is_file = e.file_type().map(|ft| ft.is_file()).unwrap_or(false);
                        if !is_file {
                            // println!("DEBUG: Skipping non-file: {:?}", e.path());
                        }
                        is_file
                    })
                    .filter_map(|e| {
                        let metadata = e.metadata().ok()?;
                        let modified = metadata.modified().ok()?;
                        // println!("DEBUG: Found file: {:?} (modified: {:?})", e.path(), modified);
                        Some((e.path(), modified))
                    })
                    .collect();

                // println!("DEBUG: Found {} files in {}", files.len(), expanded_path);
                files.sort_by(|a, b| b.1.cmp(&a.1)); // Sort by modification time, newest first

                for (path, _) in files {
                    if let Some(name) = path.file_name() {
                        let app_info = AppInfo {
                            name: format!("📄 {}", name.to_string_lossy()),
                            description: format!("Recent file: {}", path.display()),
                            exec: String::new(), // Will be handled specially
                            icon: get_file_icon(&path),
                            categories: vec!["Recent".to_string()],
                            item_type: ItemType::RecentFile,
                            file_path: Some(path.clone()),
                        };
                        // println!("DEBUG: Added directory file: {:?}", app_info.name);
                        directory_files.push(app_info);
                    }
                }
            },
            Err(e) => {
                println!("DEBUG: Error reading directory {}: {}", expanded_path, e);
            }
        }
    }

    // Load from ~/.local/share/recently-used.xbel
    let mut xbel_files = Vec::new();
    load_from_xbel(&mut xbel_files, settings);
    
    // Combine files with XBEL files taking priority
    // First, add all XBEL files
    let mut existing_paths: HashMap<PathBuf, bool> = HashMap::new();
    for xbel_file in xbel_files {
        if let Some(ref path) = xbel_file.file_path {
            existing_paths.insert(path.clone(), true);
        }
        recent_files.push(xbel_file);
    }
    
    // println!("DEBUG: Added {} XBEL files", recent_files.len());
    
    // Then add directory files that aren't already in XBEL, up to the limit
    let remaining_slots = settings.recent_files.max_files.saturating_sub(recent_files.len());
    // println!("DEBUG: Remaining slots for directory files: {}", remaining_slots);
    
    let mut added_from_dirs = 0;
    for dir_file in directory_files {
        if added_from_dirs >= remaining_slots {
            break;
        }
        
        if let Some(ref path) = dir_file.file_path {
            if !existing_paths.contains_key(path) {
                // println!("DEBUG: Adding directory file: {:?}", dir_file.name);
                recent_files.push(dir_file);
                added_from_dirs += 1;
            } else {
                // println!("DEBUG: Skipping duplicate directory file: {:?}", path);
            }
        }
    }
    
    // println!("DEBUG: Added {} directory files", added_from_dirs);
    
    // Final sort by modification time while preserving XBEL priority
    recent_files.sort_by(|a, b| {
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
    
    // println!("DEBUG: Total recent files loaded: {}", recent_files.len());
}

fn load_from_xbel(recent_files: &mut Vec<AppInfo>, settings: &LauncherSettings) {

    let xbel_path = &settings.recent_files.xbel_path;
    
    // println!("DEBUG: Loading from XBEL file: {}", xbel_path);

    match fs::read_to_string(&xbel_path) {
        Ok(content) => {
            // println!("DEBUG: XBEL file size: {} bytes", content.len());
            match parse_xbel(&content) {
                Ok(xbel_files) => {
                    // println!("DEBUG: Parsed {} files from XBEL", xbel_files.len());
                    recent_files.extend(xbel_files);
                },
                Err(_e) => {
                    // println!("DEBUG: Error parsing XBEL: {}", e);
                }
            }
        },
        Err(e) => {
            println!("DEBUG: Error reading XBEL file: {}", e);
        }
    }
}

fn parse_xbel(content: &str) -> Result<Vec<AppInfo>, Box<dyn std::error::Error>> {
    let mut files = Vec::new();
    let mut current_bookmark: Option<XbelBookmark> = None;

    // println!("DEBUG: Parsing XBEL content...");
    
    // Simple XML parsing - in production, consider using a proper XML parser like quick-xml
    for (_line_num, line) in content.lines().enumerate() {
        let line = line.trim();

        if line.starts_with("<bookmark href=\"file://") {
            // Extract file path from href
            if let Some(start) = line.find("file://") {
                if let Some(end) = line[start..].find("\"") {
                    let file_url = &line[start..start + end];
                    let file_path = file_url.replace("file://", "").replace("%20", " ");
                    // println!("DEBUG: Line {}: Found bookmark for: {}", line_num, file_path);

                    current_bookmark = Some(XbelBookmark {
                        file_path: PathBuf::from(file_path),
                        mime_type: String::new(),
                    });
                }
            }
        } else if line.contains("<mime:mime-type type=\"") {
            if let Some(bookmark) = &mut current_bookmark {
                if let Some(start) = line.find("type=\"") {
                    if let Some(end) = line[start + 6..].find("\"") {
                        bookmark.mime_type = line[start + 6..start + 6 + end].to_string();
                        // println!("DEBUG: Line {}: MIME type: {}", line_num, bookmark.mime_type);
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
                            icon: get_file_icon(&bookmark.file_path),
                            categories: vec!["Recent".to_string()],
                            item_type: ItemType::RecentFile,
                            file_path: Some(bookmark.file_path.clone()),
                        };
                        // println!("DEBUG: Adding XBEL file: {} (exists: {})", 
                                // app_info.name, bookmark.file_path.exists());
                        files.push(app_info);
                    }
                } else {
                    // println!("DEBUG: Skipping non-existent file: {:?}", bookmark.file_path);
                }
            }
        }
    }

    // println!("DEBUG: XBEL parsing complete. Found {} valid files", files.len());
    Ok(files)
}