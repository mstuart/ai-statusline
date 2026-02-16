use std::path::Path;

use super::data::SessionData;
use super::traits::{Widget, WidgetConfig, WidgetOutput};

pub struct CwdWidget;

fn get_working_dir(data: &SessionData) -> Option<String> {
    data.workspace
        .as_ref()
        .and_then(|w| w.current_dir.clone())
        .or_else(|| data.cwd.clone())
}

fn home_dir() -> Option<String> {
    std::env::var("HOME").ok()
}

fn abbreviate_home(path: &str) -> String {
    if let Some(home) = home_dir() {
        if path == home {
            return "~".to_string();
        }
        if let Some(rest) = path.strip_prefix(&home) {
            if rest.starts_with('/') {
                return format!("~{rest}");
            }
        }
    }
    path.to_string()
}

fn fish_style(path: &str) -> String {
    let abbreviated = abbreviate_home(path);

    let parts: Vec<&str> = abbreviated.split('/').collect();
    if parts.len() <= 1 {
        return abbreviated;
    }

    let mut result = Vec::new();
    for (i, part) in parts.iter().enumerate() {
        if i == parts.len() - 1 {
            // Last segment: keep full
            result.push(part.to_string());
        } else if *part == "~" {
            result.push("~".to_string());
        } else if part.is_empty() {
            // Leading slash produces empty first element
            result.push(String::new());
        } else {
            // Abbreviate to first char
            result.push(part.chars().next().unwrap().to_string());
        }
    }
    result.join("/")
}

fn last_n_segments(path: &str, n: usize) -> String {
    let abbreviated = abbreviate_home(path);
    let parts: Vec<&str> = abbreviated.split('/').collect();
    if parts.len() <= n {
        return abbreviated;
    }
    parts[parts.len() - n..].join("/")
}

impl Widget for CwdWidget {
    fn name(&self) -> &str {
        "cwd"
    }

    fn render(&self, data: &SessionData, config: &WidgetConfig) -> WidgetOutput {
        let dir = match get_working_dir(data) {
            Some(d) => d,
            None => {
                return WidgetOutput {
                    text: String::new(),
                    display_width: 0,
                    priority: 80,
                    visible: false,
                }
            }
        };

        let text = if config.metadata.get("fish_style").map(|v| v.as_str()) == Some("true") {
            fish_style(&dir)
        } else if config.metadata.get("full").map(|v| v.as_str()) == Some("true") {
            abbreviate_home(&dir)
        } else if let Some(n_str) = config.metadata.get("segments") {
            let n: usize = n_str.parse().unwrap_or(1);
            last_n_segments(&dir, n)
        } else {
            // Default: basename only
            Path::new(&dir)
                .file_name()
                .map(|f| f.to_string_lossy().to_string())
                .unwrap_or_else(|| dir.clone())
        };

        let display_width = text.len();
        WidgetOutput {
            text,
            display_width,
            priority: 80,
            visible: true,
        }
    }
}
