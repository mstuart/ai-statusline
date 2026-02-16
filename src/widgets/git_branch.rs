use std::fs;
use std::path::PathBuf;
use std::process::Command;
use std::time::SystemTime;

use super::data::SessionData;
use super::traits::{Widget, WidgetConfig, WidgetOutput};

pub struct GitBranchWidget;

fn cache_path(prefix: &str, dir: &str) -> PathBuf {
    let hash: String = dir.bytes().take(8).map(|b| format!("{:02x}", b)).collect();
    PathBuf::from(format!("/tmp/claudeline-{prefix}-{hash}"))
}

fn read_cache(path: &PathBuf, max_age_secs: u64) -> Option<String> {
    let meta = fs::metadata(path).ok()?;
    let age = SystemTime::now()
        .duration_since(meta.modified().ok()?)
        .ok()?;
    if age.as_secs() <= max_age_secs {
        fs::read_to_string(path).ok()
    } else {
        None
    }
}

fn get_working_dir(data: &SessionData) -> Option<String> {
    data.workspace
        .as_ref()
        .and_then(|w| w.current_dir.clone())
        .or_else(|| data.cwd.clone())
}

impl Widget for GitBranchWidget {
    fn name(&self) -> &str {
        "git-branch"
    }

    fn render(&self, data: &SessionData, config: &WidgetConfig) -> WidgetOutput {
        let dir = match get_working_dir(data) {
            Some(d) => d,
            None => {
                return WidgetOutput {
                    text: String::new(),
                    display_width: 0,
                    priority: 75,
                    visible: false,
                }
            }
        };

        let cache = cache_path("git-branch", &dir);

        if let Some(cached) = read_cache(&cache, 5) {
            let text = if config.raw_value {
                cached.trim().to_string()
            } else {
                cached.trim().to_string()
            };
            let display_width = text.len();
            return WidgetOutput {
                text,
                display_width,
                priority: 75,
                visible: true,
            };
        }

        // Try git branch --show-current
        let branch = Command::new("git")
            .args(["branch", "--show-current"])
            .current_dir(&dir)
            .output()
            .ok()
            .and_then(|o| {
                if o.status.success() {
                    let s = String::from_utf8_lossy(&o.stdout).trim().to_string();
                    if s.is_empty() {
                        None // detached HEAD
                    } else {
                        Some(s)
                    }
                } else {
                    None
                }
            });

        let result = match branch {
            Some(b) => b,
            None => {
                // Detached HEAD fallback
                match Command::new("git")
                    .args(["rev-parse", "--short", "HEAD"])
                    .current_dir(&dir)
                    .output()
                {
                    Ok(o) if o.status.success() => {
                        let hash = String::from_utf8_lossy(&o.stdout).trim().to_string();
                        if hash.is_empty() {
                            return WidgetOutput {
                                text: String::new(),
                                display_width: 0,
                                priority: 75,
                                visible: false,
                            };
                        }
                        hash
                    }
                    _ => {
                        return WidgetOutput {
                            text: String::new(),
                            display_width: 0,
                            priority: 75,
                            visible: false,
                        };
                    }
                }
            }
        };

        // Write cache
        let _ = fs::write(&cache, &result);

        let display_width = result.len();
        WidgetOutput {
            text: result,
            display_width,
            priority: 75,
            visible: true,
        }
    }
}
