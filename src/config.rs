use gpui::{rgb, Rgba};
use serde::Deserialize;
use std::{
    error::Error,
    fs,
    path::{Path, PathBuf},
};

#[derive(Debug, Deserialize)]
pub struct KeyConfig {
    pub key: String,
}

#[derive(Debug, Deserialize)]
#[serde(default)]
pub struct Config {
    pub bg_color: Rgba,
    #[serde(rename = "Keys")]
    pub keys: Vec<KeyConfig>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            bg_color: rgb(0x505050),
            keys: Vec::new(),
        }
    }
}

impl Config {
    pub fn load() -> Self {
        default_config_path()
            .and_then(|p| Self::load_from_path(&p).ok())
            .unwrap_or_default()
    }

    pub fn load_from_path(path: &Path) -> Result<Self, Box<dyn Error>> {
        let s = fs::read_to_string(path)?;
        let cfg = toml::from_str::<Config>(&s)?;
        Ok(cfg)
    }
}

fn default_config_path() -> Option<PathBuf> {
    dirs_home().map(|home| home.join(".config").join("key-display").join("config.toml"))
}

fn dirs_home() -> Option<PathBuf> {
    std::env::var_os("HOME").map(PathBuf::from)
}
