use std::path::Path;
use std::process::Command;

use super::data::SessionData;
use super::traits::{Widget, WidgetConfig, WidgetOutput};

pub struct GitWorktreeWidget;

fn get_working_dir(data: &SessionData) -> Option<String> {
    data.workspace
        .as_ref()
        .and_then(|w| w.current_dir.clone())
        .or_else(|| data.cwd.clone())
}

impl Widget for GitWorktreeWidget {
    fn name(&self) -> &str {
        "git-worktree"
    }

    fn render(&self, data: &SessionData, config: &WidgetConfig) -> WidgetOutput {
        let dir = match get_working_dir(data) {
            Some(d) => d,
            None => {
                return WidgetOutput {
                    text: String::new(),
                    display_width: 0,
                    priority: 45,
                    visible: false,
                }
            }
        };

        let toplevel = Command::new("git")
            .args(["rev-parse", "--show-toplevel"])
            .current_dir(&dir)
            .output()
            .ok()
            .and_then(|o| {
                if o.status.success() {
                    Some(String::from_utf8_lossy(&o.stdout).trim().to_string())
                } else {
                    None
                }
            });

        let git_common_dir = Command::new("git")
            .args(["rev-parse", "--git-common-dir"])
            .current_dir(&dir)
            .output()
            .ok()
            .and_then(|o| {
                if o.status.success() {
                    Some(String::from_utf8_lossy(&o.stdout).trim().to_string())
                } else {
                    None
                }
            });

        let (toplevel, git_common_dir) = match (toplevel, git_common_dir) {
            (Some(t), Some(g)) => (t, g),
            _ => {
                return WidgetOutput {
                    text: String::new(),
                    display_width: 0,
                    priority: 45,
                    visible: false,
                }
            }
        };

        // Resolve git_common_dir relative to toplevel if it's relative
        let common_resolved = if Path::new(&git_common_dir).is_relative() {
            Path::new(&toplevel)
                .join(&git_common_dir)
                .canonicalize()
                .unwrap_or_else(|_| Path::new(&git_common_dir).to_path_buf())
        } else {
            Path::new(&git_common_dir)
                .canonicalize()
                .unwrap_or_else(|_| Path::new(&git_common_dir).to_path_buf())
        };

        // .git dir for the toplevel
        let toplevel_git = Path::new(&toplevel).join(".git");
        let toplevel_git_resolved = toplevel_git
            .canonicalize()
            .unwrap_or_else(|_| toplevel_git.clone());

        // If common dir differs from the toplevel's .git, this is a worktree
        let is_worktree = common_resolved != toplevel_git_resolved
            && git_common_dir != ".git"
            && git_common_dir != format!("{}/.git", toplevel);

        if !is_worktree {
            return WidgetOutput {
                text: String::new(),
                display_width: 0,
                priority: 45,
                visible: false,
            };
        }

        let folder_name = Path::new(&toplevel)
            .file_name()
            .map(|f| f.to_string_lossy().to_string())
            .unwrap_or_default();

        let text = if config.raw_value {
            folder_name.clone()
        } else {
            format!("WT: {folder_name}")
        };
        let display_width = text.len();

        WidgetOutput {
            text,
            display_width,
            priority: 45,
            visible: true,
        }
    }
}
