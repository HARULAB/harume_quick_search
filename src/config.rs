use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::i18n::Language;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AppConfig {
    pub width: f32,
    pub height: f32,
    pub use_counts: HashMap<String, u32>,
    #[serde(default)]
    pub language: Language,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            width: 850.0,
            height: 900.0,
            use_counts: HashMap::new(),
            language: Language::Japanese,
        }
    }
}

pub fn get_config_path() -> std::path::PathBuf {
    let mut path = std::path::PathBuf::from(r"C:\ProgramData\aviutl2\Plugin");
    let _ = std::fs::create_dir_all(&path);
    path.push("quick_search_settings.json");
    path
}

pub fn load_config() -> AppConfig {
    std::fs::read_to_string(get_config_path())
        .ok()
        .and_then(|s| serde_json::from_str(&s).ok())
        .unwrap_or_default()
}

pub fn save_config(config: &AppConfig) {
    if let Ok(s) = serde_json::to_string(config) {
        let _ = std::fs::write(get_config_path(), s);
    }
}
