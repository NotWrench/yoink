use anyhow::{Context, Result};
use directories::ProjectDirs;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Deserialize, Serialize, Default)]
pub struct Config {
    #[serde(default)]
    pub projects: HashMap<String, ProjectConfig>,
}

#[derive(Debug, Deserialize, Serialize, Default, Clone)]
pub struct ProjectConfig {
    #[serde(default)]
    pub bound_profile: Option<String>,
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
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_file_size: Option<usize>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub depth: Option<usize>,
}

pub fn get_config_path() -> Result<PathBuf> {
    let proj_dirs =
        ProjectDirs::from("", "", "yoink").context("Could not determine config directory")?;
    let config_dir = proj_dirs.config_dir();
    fs::create_dir_all(config_dir).context("Failed to create config directory")?;
    Ok(config_dir.join("config.toml"))
}

pub fn load_config() -> Result<Config> {
    let path = get_config_path()?;
    if path.exists() {
        let content = fs::read_to_string(path).context("Failed to read config.toml")?;
        let config = toml::from_str(&content).context("Failed to parse config.toml")?;
        Ok(config)
    } else {
        Ok(Config::default())
    }
}

pub fn save_config(config: &Config) -> Result<()> {
    let path = get_config_path()?;
    let content = toml::to_string_pretty(config).context("Failed to serialize config")?;
    fs::write(path, content).context("Failed to write config file")?;
    Ok(())
}
