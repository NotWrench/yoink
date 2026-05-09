use directories::ProjectDirs;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Deserialize, Serialize, Default)]
pub struct Config {
    #[serde(default)]
    pub bindings: HashMap<String, String>,
    #[serde(default)]
    pub profiles: HashMap<String, Profile>,
}

#[derive(Debug, Deserialize, Serialize, Default, Clone)]
pub struct Profile {
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub exclude: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub include_only: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub include_hidden: Vec<String>,
}

pub fn get_config_path() -> PathBuf {
    let proj_dirs =
        ProjectDirs::from("", "", "yoink").expect("Could not determine config directory");
    let config_dir = proj_dirs.config_dir();
    fs::create_dir_all(config_dir).expect("Failed to create config directory");
    config_dir.join("config.toml")
}

pub fn load_config() -> Config {
    let path = get_config_path();
    if path.exists() {
        let content = fs::read_to_string(path).unwrap_or_default();
        toml::from_str(&content).unwrap_or_default()
    } else {
        Config::default()
    }
}

pub fn save_config(config: &Config) {
    let path = get_config_path();
    let content = toml::to_string_pretty(config).expect("Failed to serialize config");
    fs::write(path, content).expect("Failed to write config file");
}
