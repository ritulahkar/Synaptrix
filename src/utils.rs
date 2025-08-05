use gtk::Image;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::fs;

pub fn launch_application(exec: &str) {
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

pub fn open_file(path: &Path) {
    // Use xdg-open to open files with default applications
    Command::new("xdg-open")
        .arg(path)
        .spawn()
        .unwrap_or_else(|e| {
            eprintln!("Failed to open file: {}", e);
            std::process::exit(1);
        });
}

pub fn is_image_file(path: &Path) -> bool {
    if let Some(ext) = path.extension() {
        matches!(
            ext.to_str().unwrap_or("").to_lowercase().as_str(),
            "png" | "jpg" | "jpeg" | "gif" | "bmp" | "tiff" | "webp" | "svg"
        )
    } else {
        false
    }
}

pub fn create_thumbnail(path: &Path, size: i32) -> Result<PathBuf, Box<dyn std::error::Error>> {
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

pub fn get_file_icon(path: &Path) -> Option<String> {
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

// pub fn find_and_click_button(widget: &gtk::Widget) {
//     if let Some(button) = widget.downcast_ref::<Button>() {
//         button.emit_clicked();
//         return;
//     }
//     if let Some(container) = widget.downcast_ref::<GtkBox>() {
//         let mut child = container.first_child();
//         while let Some(current_child) = child {
//             find_and_click_button(&current_child);
//             child = current_child.next_sibling();
//         }
//     }
// }

pub fn create_icon_widget(
    icon_name: &Option<String>,
    item_type: &crate::app_info::ItemType,
    file_path: &Option<PathBuf>,
) -> Image {
    use crate::app_info::ItemType;
    use gtk::gdk;
    use gtk::prelude::*;

    let icon = Image::new();
    icon.set_pixel_size(48);
    icon.set_valign(gtk::Align::Center);

    // For image files, try to create thumbnails
    if let Some(path) = file_path {
        if is_image_file(path) {
            if let Ok(thumbnail) = create_thumbnail(path, 48) {
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