use std::collections::HashMap;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use crate::widgets::WidgetConfig;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    #[serde(default = "default_lines")]
    pub lines: Vec<Vec<LineWidgetConfig>>,
    #[serde(default = "default_theme")]
    pub theme: String,
    #[serde(default)]
    pub powerline: PowerlineConfig,
    #[serde(default = "default_color_level")]
    pub color_level: String,
    #[serde(default = "default_padding")]
    pub default_padding: String,
    #[serde(default = "default_flex_mode")]
    pub flex_mode: String,
    #[serde(default = "default_compact_threshold")]
    pub compact_threshold: u8,
    #[serde(default)]
    pub global_bold: bool,
    #[serde(default)]
    pub inherit_separator_colors: bool,
    #[serde(default = "default_separator")]
    pub default_separator: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LineWidgetConfig {
    #[serde(rename = "type")]
    pub widget_type: String,
    #[serde(default)]
    pub id: String,
    pub color: Option<String>,
    pub background_color: Option<String>,
    pub bold: Option<bool>,
    #[serde(default)]
    pub raw_value: bool,
    pub padding: Option<String>,
    #[serde(default)]
    pub merge_next: bool,
    #[serde(default)]
    pub metadata: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PowerlineConfig {
    #[serde(default)]
    pub enabled: bool,
    #[serde(default = "default_powerline_separator")]
    pub separator: String,
    #[serde(default)]
    pub separator_invert_background: bool,
    #[serde(default)]
    pub start_cap: Option<String>,
    #[serde(default)]
    pub end_cap: Option<String>,
    #[serde(default)]
    pub auto_align: bool,
}

impl Default for PowerlineConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            separator: default_powerline_separator(),
            separator_invert_background: false,
            start_cap: None,
            end_cap: None,
            auto_align: false,
        }
    }
}

fn default_lines() -> Vec<Vec<LineWidgetConfig>> {
    vec![vec![
        LineWidgetConfig {
            widget_type: "model".into(),
            id: "1".into(),
            color: Some("cyan".into()),
            background_color: None,
            bold: None,
            raw_value: false,
            padding: None,
            merge_next: false,
            metadata: HashMap::new(),
        },
        LineWidgetConfig {
            widget_type: "context-percentage".into(),
            id: "2".into(),
            color: None,
            background_color: None,
            bold: None,
            raw_value: false,
            padding: None,
            merge_next: false,
            metadata: HashMap::new(),
        },
        LineWidgetConfig {
            widget_type: "session-cost".into(),
            id: "3".into(),
            color: Some("yellow".into()),
            background_color: None,
            bold: None,
            raw_value: true,
            padding: None,
            merge_next: false,
            metadata: HashMap::new(),
        },
        LineWidgetConfig {
            widget_type: "session-duration".into(),
            id: "4".into(),
            color: None,
            background_color: None,
            bold: None,
            raw_value: true,
            padding: None,
            merge_next: false,
            metadata: HashMap::new(),
        },
    ]]
}

fn default_theme() -> String { "default".into() }
fn default_color_level() -> String { "auto".into() }
fn default_padding() -> String { " ".into() }
fn default_flex_mode() -> String { "full-minus-40".into() }
fn default_compact_threshold() -> u8 { 60 }
fn default_separator() -> String { " | ".into() }
fn default_powerline_separator() -> String { "\u{E0B0}".into() }

impl Config {
    pub fn load(path: Option<&str>) -> Self {
        let config_path = path
            .map(PathBuf::from)
            .or_else(Self::default_path);

        match config_path {
            Some(p) if p.exists() => {
                let contents = std::fs::read_to_string(&p).unwrap_or_default();
                toml::from_str(&contents).unwrap_or_default()
            }
            _ => Self::default(),
        }
    }

    pub fn default_path() -> Option<PathBuf> {
        // Check CLAUDE_CONFIG_DIR first
        if let Ok(dir) = std::env::var("CLAUDE_CONFIG_DIR") {
            let p = PathBuf::from(dir).join("claudeline").join("config.toml");
            if p.exists() {
                return Some(p);
            }
        }
        // XDG config
        dirs::config_dir().map(|d| d.join("claudeline").join("config.toml"))
    }

    pub fn to_toml(&self) -> String {
        toml::to_string_pretty(self).unwrap_or_default()
    }

    pub fn to_widget_config(lwc: &LineWidgetConfig) -> WidgetConfig {
        WidgetConfig {
            widget_type: lwc.widget_type.clone(),
            id: lwc.id.clone(),
            color: lwc.color.clone(),
            background_color: lwc.background_color.clone(),
            bold: lwc.bold,
            raw_value: lwc.raw_value,
            padding: lwc.padding.clone(),
            merge_next: lwc.merge_next,
            metadata: lwc.metadata.clone(),
        }
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            lines: default_lines(),
            theme: default_theme(),
            powerline: PowerlineConfig::default(),
            color_level: default_color_level(),
            default_padding: default_padding(),
            flex_mode: default_flex_mode(),
            compact_threshold: default_compact_threshold(),
            global_bold: false,
            inherit_separator_colors: false,
            default_separator: default_separator(),
        }
    }
}
