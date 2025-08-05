use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct AppInfo {
    pub name: String,
    pub description: String,
    pub exec: String,
    pub icon: Option<String>,
    pub categories: Vec<String>,
    pub item_type: ItemType,
    pub file_path: Option<PathBuf>, // Added for file handling
}

#[derive(Debug, Clone, PartialEq)]
pub enum ItemType {
    Application,
    Command,
    RecentFile,
}

#[derive(Debug)]
pub struct XbelBookmark {
    pub file_path: PathBuf,
    pub mime_type: String,
}