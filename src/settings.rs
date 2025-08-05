use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LauncherSettings {
    pub window: WindowSettings,
    pub theme: ThemeSettings,
    pub behavior: BehaviorSettings,
    pub recent_files: RecentFilesSettings,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WindowSettings {
    pub width: i32,
    pub height: i32,
    pub position: String, // "center", "top", "bottom"
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThemeSettings {
    pub background_color: String,
    pub accent_color: String,
    pub text_color: String,
    pub transparency: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BehaviorSettings {
    pub max_results: usize,
    pub auto_close: bool,
    pub show_descriptions: bool,
    pub quit_on_close: bool, // New setting: true = quit app, false = stay in memory
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecentFilesSettings {
    pub enabled: bool,
    pub max_files: usize,
    pub directories: Vec<String>,
    pub xbel_path: String,
}

impl Default for LauncherSettings {
    fn default() -> Self {
        Self {
            window: WindowSettings {
                width: 700,
                height: 500,
                position: "center".to_string(),
            },
            theme: ThemeSettings {
                background_color: "rgba(248, 249, 250, 0.70)".to_string(),
                accent_color: "rgba(52, 152, 219, 0.8)".to_string(),
                text_color: "#2c3e50".to_string(),
                transparency: 0.95,
            },
            behavior: BehaviorSettings {
                max_results: 50,
                auto_close: true,
                show_descriptions: true,
                quit_on_close: false, // Default to staying in memory (daemon mode)
            },
            recent_files: RecentFilesSettings {
                enabled: true,
                max_files: 200,
                xbel_path: "~/.local/share/recently-used.xbel".to_string(),
                directories: vec![
                    "~/Documents".to_string(),
                    "~/Downloads".to_string(),
                    "~/Desktop".to_string(),
                    "~/Pictures".to_string(),
                ],
            },
        }
    }
}

impl LauncherSettings {
    /// Expand ~ to home directory in a path string
    fn expand_tilde(path: &str) -> Result<String, Box<dyn std::error::Error>> {
        if path.starts_with('~') {
            let home = std::env::var("HOME")
                .map_err(|_| "Unable to determine HOME directory")?;
            
            if path == "~" {
                Ok(home)
            } else if path.starts_with("~/") {
                Ok(path.replacen('~', &home, 1))
            } else {
                // Path starts with ~ but not ~/ (like ~username), leave as is
                Ok(path.to_string())
            }
        } else {
            Ok(path.to_string())
        }
    }

    /// Expand ~ in all path fields of the settings
    fn expand_paths(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        // Expand xbel_path
        self.recent_files.xbel_path = Self::expand_tilde(&self.recent_files.xbel_path)?;
        
        // Expand directories
        let mut expanded_dirs = Vec::new();
        for dir in &self.recent_files.directories {
            expanded_dirs.push(Self::expand_tilde(dir)?);
        }
        self.recent_files.directories = expanded_dirs;
        
        Ok(())
    }

    /// Get the path to the config file
    fn config_path() -> Result<PathBuf, Box<dyn std::error::Error>> {
        let home = std::env::var("HOME")
            .map_err(|_| "Unable to determine HOME directory")?;
        
        let config_dir = PathBuf::from(home).join(".config").join("synaptrix");
        
        // Create config directory if it doesn't exist
        if !config_dir.exists() {
            fs::create_dir_all(&config_dir)?;
        }

        Ok(config_dir.join("settings.yaml"))
    }

    /// Load settings from YAML file, create default file if it doesn't exist
    pub fn load() -> Self {
        match Self::load_from_file() {
            Ok(settings) => settings, // paths are already expanded in load_from_file
            Err(_) => {
                // File doesn't exist, create default settings file
                Self::create_default_with_expanded_paths()
            }
        }
    }

    /// Create default settings, save to file with ~ paths, return with expanded paths
    fn create_default_with_expanded_paths() -> Self {
        let default_settings = Self::default();
        
        // Save the default settings (with ~ paths) to file
        if let Err(e) = default_settings.save() {
            eprintln!("Warning: Could not save default settings file: {}", e);
        }
        
        // Return expanded version for use in the application
        let mut expanded_settings = default_settings;
        if let Err(e) = expanded_settings.expand_paths() {
            eprintln!("Warning: Could not expand paths in default settings: {}", e);
        }
        
        expanded_settings
    }

    /// Load settings from YAML file
    fn load_from_file() -> Result<Self, Box<dyn std::error::Error>> {
        let config_path = Self::config_path()?;
        
        if !config_path.exists() {
            return Err("Config file does not exist".into());
        }

        let content = fs::read_to_string(&config_path)?;
        let mut settings: LauncherSettings = serde_yaml::from_str(&content)?;
        
        // Always expand paths when loading from file, in case file contains ~ paths
        settings.expand_paths()?;
        
        Ok(settings)
    }

    /// Save settings to YAML file
    pub fn save(&self) -> Result<(), Box<dyn std::error::Error>> {
        let config_path = Self::config_path()?;
        
        let yaml_content = serde_yaml::to_string(self)?;
        fs::write(&config_path, yaml_content)?;
        
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_expand_tilde() {
        std::env::set_var("HOME", "/home/testuser");
        
        assert_eq!(
            LauncherSettings::expand_tilde("~/Documents").unwrap(),
            "/home/testuser/Documents"
        );
        assert_eq!(
            LauncherSettings::expand_tilde("/absolute/path").unwrap(),
            "/absolute/path"
        );
    }

    #[test]
    fn test_save_and_load() {
        // Create a temporary directory for testing
        let temp_dir = tempdir().unwrap();
        let config_path = temp_dir.path().join("test_settings.yaml");
        
        // Create test settings
        let mut settings = LauncherSettings::default();
        settings.window.width = 800;
        settings.theme.background_color = "red".to_string();
        settings.behavior.quit_on_close = true;
        
        // Save to YAML
        let yaml_content = serde_yaml::to_string(&settings).unwrap();
        fs::write(&config_path, yaml_content).unwrap();
        
        // Load from YAML
        let content = fs::read_to_string(&config_path).unwrap();
        let loaded_settings: LauncherSettings = serde_yaml::from_str(&content).unwrap();
        
        // Verify the data matches
        assert_eq!(loaded_settings.window.width, 800);
        assert_eq!(loaded_settings.theme.background_color, "red");
        assert_eq!(loaded_settings.behavior.quit_on_close, true);
    }

    #[test]
    fn test_default_settings() {
        let settings = LauncherSettings::default();
        assert_eq!(settings.window.width, 700);
        assert_eq!(settings.behavior.max_results, 50);
        assert!(settings.recent_files.enabled);
        assert_eq!(settings.behavior.quit_on_close, false); // Default is daemon mode
    }

    #[test]
    fn test_path_expansion() {
        std::env::set_var("HOME", "/home/testuser");
        
        let mut settings = LauncherSettings::default();
        settings.expand_paths().unwrap();
        
        assert_eq!(settings.recent_files.xbel_path, "/home/testuser/.local/share/recently-used.xbel");
        assert!(settings.recent_files.directories.contains(&"/home/testuser/Documents".to_string()));
    }
}