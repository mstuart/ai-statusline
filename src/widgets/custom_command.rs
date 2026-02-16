use super::traits::{Widget, WidgetConfig, WidgetOutput};
use super::data::SessionData;
use std::fs;
use std::process::Command;
use std::time::{SystemTime, Duration};

pub struct CustomCommandWidget;

fn cache_path(command: &str) -> std::path::PathBuf {
    let hash: String = command.bytes().take(16).map(|b| format!("{:02x}", b)).collect();
    std::path::PathBuf::from(format!("/tmp/claudeline-cmd-{hash}"))
}

fn read_cache(path: &std::path::Path) -> Option<String> {
    let metadata = fs::metadata(path).ok()?;
    let modified = metadata.modified().ok()?;
    if SystemTime::now().duration_since(modified).ok()? > Duration::from_secs(10) {
        return None;
    }
    fs::read_to_string(path).ok()
}

fn run_command(cmd: &str) -> Option<String> {
    let child = Command::new("/bin/sh")
        .arg("-c")
        .arg(cmd)
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::null())
        .spawn()
        .ok()?;

    let output = child.wait_with_output().ok()?;

    if !output.status.success() {
        return None;
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let first_line = stdout.lines().next()?.trim().to_string();
    if first_line.is_empty() {
        None
    } else {
        Some(first_line)
    }
}

impl Widget for CustomCommandWidget {
    fn name(&self) -> &str { "custom-command" }

    fn render(&self, _data: &SessionData, config: &WidgetConfig) -> WidgetOutput {
        let cmd = match config.metadata.get("command") {
            Some(c) if !c.is_empty() => c,
            _ => return WidgetOutput { text: String::new(), display_width: 0, priority: 40, visible: false },
        };

        let path = cache_path(cmd);
        let text = if let Some(cached) = read_cache(&path) {
            cached
        } else {
            match run_command(cmd) {
                Some(result) => {
                    let _ = fs::write(&path, &result);
                    result
                }
                None => return WidgetOutput { text: String::new(), display_width: 0, priority: 40, visible: false },
            }
        };

        let display_width = text.len();
        WidgetOutput { text, display_width, priority: 40, visible: true }
    }
}
