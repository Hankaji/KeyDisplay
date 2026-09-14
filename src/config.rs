use gpui::{rgb, DefiniteLength, Rgba, px};
use serde::{Deserialize, Deserializer};
use std::{
    error::Error,
    fs,
    path::{Path, PathBuf},
    str::FromStr,
};

#[derive(Debug)]
pub enum KeyWidth {
    Px(f32),
    Pct(f32),
}

impl KeyWidth {
    pub fn to_definite_length(&self) -> DefiniteLength {
        match self {
            KeyWidth::Px(v) => px(*v).into(),
            KeyWidth::Pct(v) => px(v * 64.0).into(),
        }
    }
}

impl FromStr for KeyWidth {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if let Some(inner) = s.strip_prefix("Px(").and_then(|s| s.strip_suffix(')')) {
            inner.trim().parse::<f32>().map(KeyWidth::Px).map_err(|e| e.to_string())
        } else if let Some(inner) = s.strip_prefix("Pct(").and_then(|s| s.strip_suffix(')')) {
            inner.trim().parse::<f32>().map(KeyWidth::Pct).map_err(|e| e.to_string())
        } else {
            Err(format!("invalid width '{s}', expected Px(n) or Pct(n)"))
        }
    }
}

impl<'de> Deserialize<'de> for KeyWidth {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        String::deserialize(deserializer)?
            .parse()
            .map_err(serde::de::Error::custom)
    }
}

fn default_key_width() -> KeyWidth {
    KeyWidth::Px(64.0)
}

fn default_bar_color() -> Rgba {
    rgb(0x808080)
}

#[derive(Debug, Deserialize)]
pub struct KeyConfig {
    pub key: String,
    #[serde(default = "default_key_width")]
    pub width: KeyWidth,
    #[serde(default = "default_bar_color")]
    pub bar_color: Rgba,
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
