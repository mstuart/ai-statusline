use std::fs;
use std::path::PathBuf;
use std::process::Command;
use std::time::SystemTime;

use super::data::SessionData;
use super::traits::{Widget, WidgetConfig, WidgetOutput};

pub struct GitStatusWidget;

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

fn format_status(staged: usize, modified: usize, untracked: usize) -> String {
    let mut parts = Vec::new();
    if staged > 0 {
        parts.push(format!("+{staged}"));
    }
    if modified > 0 {
        parts.push(format!("~{modified}"));
    }
    if untracked > 0 {
        parts.push(format!("?{untracked}"));
    }
    parts.join(" ")
}

impl Widget for GitStatusWidget {
    fn name(&self) -> &str {
        "git-status"
    }

    fn render(&self, data: &SessionData, _config: &WidgetConfig) -> WidgetOutput {
        let dir = match get_working_dir(data) {
            Some(d) => d,
            None => {
                return WidgetOutput {
                    text: String::new(),
                    display_width: 0,
                    priority: 70,
                    visible: false,
                }
            }
        };

        let cache = cache_path("git-status", &dir);

        if let Some(cached) = read_cache(&cache, 5) {
            let text = cached.trim().to_string();
            if text.is_empty() {
                return WidgetOutput {
                    text: String::new(),
                    display_width: 0,
                    priority: 70,
                    visible: true,
                };
            }
            let display_width = text.len();
            return WidgetOutput {
                text,
                display_width,
                priority: 70,
                visible: true,
            };
        }

        let output = match Command::new("git")
            .args(["status", "--porcelain"])
            .current_dir(&dir)
            .output()
        {
            Ok(o) if o.status.success() => String::from_utf8_lossy(&o.stdout).to_string(),
            _ => {
                return WidgetOutput {
                    text: String::new(),
                    display_width: 0,
                    priority: 70,
                    visible: false,
                };
            }
        };

        let mut staged = 0usize;
        let mut modified = 0usize;
        let mut untracked = 0usize;

        for line in output.lines() {
            let bytes = line.as_bytes();
            if bytes.len() < 2 {
                continue;
            }
            let index = bytes[0];
            let worktree = bytes[1];

            if index == b'?' && worktree == b'?' {
                untracked += 1;
            } else {
                // Index column: staged changes
                if matches!(index, b'A' | b'M' | b'D' | b'R') {
                    staged += 1;
                }
                // Working tree column: modified/deleted
                if matches!(worktree, b'M' | b'D') {
                    modified += 1;
                }
            }
        }

        let text = format_status(staged, modified, untracked);

        // Write cache
        let _ = fs::write(&cache, &text);

        let display_width = text.len();
        WidgetOutput {
            text,
            display_width,
            priority: 70,
            visible: true,
        }
    }
}
