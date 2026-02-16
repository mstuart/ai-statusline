use std::collections::HashMap;

use clap::Subcommand;

use claudeline::config::{Config, LineWidgetConfig, PowerlineConfig};
use claudeline::themes::Theme;

#[derive(Subcommand)]
pub enum Commands {
    /// Launch interactive TUI configuration
    Config,
    /// Generate default config file
    Init,
    /// Check environment compatibility
    Doctor,
    /// Manage themes
    Theme {
        #[command(subcommand)]
        action: ThemeAction,
    },
    /// Apply a preset layout
    Preset {
        /// Preset name: minimal, full, powerline, compact
        name: String,
    },
    /// Dump the expected JSON input schema
    DumpSchema,
}

#[derive(Subcommand)]
pub enum ThemeAction {
    /// List available themes
    List,
    /// Set active theme
    Set { name: String },
}

pub fn handle_command(cmd: Commands) {
    match cmd {
        Commands::Config => {
            eprintln!("TUI config not yet implemented");
        }
        Commands::Init => cmd_init(),
        Commands::Doctor => cmd_doctor(),
        Commands::Theme { action } => match action {
            ThemeAction::List => cmd_theme_list(),
            ThemeAction::Set { name } => cmd_theme_set(&name),
        },
        Commands::Preset { name } => cmd_preset(&name),
        Commands::DumpSchema => cmd_dump_schema(),
    }
}

fn config_path() -> std::path::PathBuf {
    dirs::config_dir()
        .unwrap_or_else(|| std::path::PathBuf::from(".config"))
        .join("claudeline")
        .join("config.toml")
}

fn cmd_init() {
    let path = config_path();
    if let Some(parent) = path.parent() {
        if let Err(e) = std::fs::create_dir_all(parent) {
            eprintln!("Error creating config directory: {e}");
            return;
        }
    }

    let config = Config::default();
    let toml_str = config.to_toml();

    if let Err(e) = std::fs::write(&path, &toml_str) {
        eprintln!("Error writing config file: {e}");
        return;
    }

    println!("Config written to: {}", path.display());
    println!();
    println!("{toml_str}");
    println!("---");
    println!("To use with Claude Code, add to your settings.json:");
    println!();
    println!(r#"  "preferences": {{"#);
    println!(r#"    "statusline": {{"#);
    println!(r#"      "command": "claudeline""#);
    println!(r#"    }}"#);
    println!(r#"  }}"#);
}

fn cmd_doctor() {
    println!("claudeline doctor");
    println!("=================");
    println!();

    // Terminal color support
    let colorterm = std::env::var("COLORTERM").unwrap_or_default();
    let term = std::env::var("TERM").unwrap_or_default();
    let color_support = if colorterm == "truecolor" || colorterm == "24bit" {
        "truecolor (24-bit)"
    } else if term.contains("256color") {
        "256 colors"
    } else if std::env::var("NO_COLOR").is_ok() {
        "none (NO_COLOR set)"
    } else {
        "basic (16 colors)"
    };
    print_check(true, &format!("Color support: {color_support}"));

    // Terminal width
    let width = crossterm::terminal::size().map(|(w, _)| w).unwrap_or(0);
    print_check(width > 0, &format!("Terminal width: {width} columns"));

    // Git availability
    let git_ok = std::process::Command::new("git")
        .arg("--version")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false);
    print_check(git_ok, "Git: available");
    if !git_ok {
        println!("   Git is not found in PATH");
    }

    // Nerd Font detection
    let nerd_hint = std::env::var("NERD_FONT").is_ok() || std::env::var("NERDFONTS").is_ok();
    if nerd_hint {
        print_check(true, "Nerd Fonts: detected via env var");
    } else {
        println!(
            "  ? Nerd Fonts: unknown (set NERD_FONT=1 to confirm, or check your terminal font)"
        );
    }

    // Config file
    let cfg_path = config_path();
    let cfg_exists = cfg_path.exists();
    if cfg_exists {
        match std::fs::read_to_string(&cfg_path) {
            Ok(contents) => {
                let valid = toml::from_str::<Config>(&contents).is_ok();
                print_check(
                    valid,
                    &format!("Config: {} (valid: {})", cfg_path.display(), valid),
                );
            }
            Err(e) => {
                print_check(
                    false,
                    &format!("Config: {} (read error: {e})", cfg_path.display()),
                );
            }
        }
    } else {
        println!(
            "  - Config: not found at {} (run `claudeline init` to create)",
            cfg_path.display()
        );
    }

    println!();
    println!("Powerline separator test: \u{E0B0} \u{E0B2}");
    println!("If the above shows triangles, your font supports powerline glyphs.");
}

fn print_check(ok: bool, msg: &str) {
    if ok {
        println!("  [ok] {msg}");
    } else {
        println!("  [!!] {msg}");
    }
}

fn cmd_theme_list() {
    println!("Available themes:");
    for name in Theme::list() {
        println!("  {name}");
    }
}

fn cmd_theme_set(name: &str) {
    let available = Theme::list();
    if !available.contains(&name) {
        eprintln!(
            "Unknown theme '{name}'. Available: {}",
            available.join(", ")
        );
        return;
    }

    let path = config_path();
    let mut config = if path.exists() {
        let contents = std::fs::read_to_string(&path).unwrap_or_default();
        toml::from_str::<Config>(&contents).unwrap_or_default()
    } else {
        Config::default()
    };

    config.theme = name.to_string();

    if let Some(parent) = path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    match std::fs::write(&path, config.to_toml()) {
        Ok(_) => println!("Theme set to '{name}' in {}", path.display()),
        Err(e) => eprintln!("Error saving config: {e}"),
    }
}

fn cmd_preset(name: &str) {
    let config = match name {
        "minimal" => preset_minimal(),
        "full" => preset_full(),
        "powerline" => preset_powerline(),
        "compact" => preset_compact(),
        _ => {
            eprintln!("Unknown preset '{name}'. Available: minimal, full, powerline, compact");
            return;
        }
    };

    let path = config_path();
    if let Some(parent) = path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    match std::fs::write(&path, config.to_toml()) {
        Ok(_) => {
            println!("Preset '{name}' written to {}", path.display());
            println!();
            println!("{}", config.to_toml());
        }
        Err(e) => eprintln!("Error saving config: {e}"),
    }
}

fn widget(widget_type: &str) -> LineWidgetConfig {
    LineWidgetConfig {
        widget_type: widget_type.into(),
        id: String::new(),
        color: None,
        background_color: None,
        bold: None,
        raw_value: false,
        padding: None,
        merge_next: false,
        metadata: HashMap::new(),
    }
}

fn widget_raw(widget_type: &str) -> LineWidgetConfig {
    let mut w = widget(widget_type);
    w.raw_value = true;
    w
}

fn widget_colored(widget_type: &str, fg: Option<&str>, bg: Option<&str>) -> LineWidgetConfig {
    let mut w = widget(widget_type);
    w.color = fg.map(String::from);
    w.background_color = bg.map(String::from);
    w
}

fn preset_minimal() -> Config {
    Config {
        lines: vec![vec![widget("model"), widget("context-percentage")]],
        ..Config::default()
    }
}

fn preset_full() -> Config {
    Config {
        lines: vec![
            vec![
                widget("model"),
                widget("context-percentage"),
                widget("tokens-input"),
                widget("tokens-output"),
                widget("session-cost"),
                widget("session-duration"),
            ],
            vec![
                widget("cwd"),
                widget("git-branch"),
                widget("git-status"),
                widget("lines-changed"),
                widget("version"),
            ],
        ],
        ..Config::default()
    }
}

fn preset_powerline() -> Config {
    Config {
        lines: vec![
            vec![
                widget_colored("model", Some("white"), Some("blue")),
                widget_colored("context-percentage", Some("white"), Some("green")),
                widget_colored("tokens-input", Some("white"), Some("cyan")),
                widget_colored("tokens-output", Some("white"), Some("magenta")),
                widget_colored("session-cost", Some("white"), Some("yellow")),
                widget_colored("session-duration", Some("white"), Some("red")),
            ],
            vec![
                widget_colored("cwd", Some("white"), Some("blue")),
                widget_colored("git-branch", Some("white"), Some("magenta")),
                widget_colored("git-status", Some("white"), Some("green")),
                widget_colored("lines-changed", Some("white"), Some("cyan")),
                widget_colored("version", Some("white"), Some("brightBlack")),
            ],
        ],
        powerline: PowerlineConfig {
            enabled: true,
            separator: "\u{E0B0}".into(),
            separator_invert_background: false,
            start_cap: None,
            end_cap: Some("\u{E0B0}".into()),
            auto_align: true,
        },
        ..Config::default()
    }
}

fn preset_compact() -> Config {
    Config {
        lines: vec![vec![
            widget_raw("model"),
            widget_raw("context-percentage"),
            widget_raw("session-cost"),
            widget_raw("session-duration"),
        ]],
        ..Config::default()
    }
}

fn cmd_dump_schema() {
    let sample = serde_json::json!({
        "cwd": "/home/user/project",
        "session_id": "abc-123-def-456",
        "transcript_path": "/tmp/claude/transcript.jsonl",
        "model": {
            "id": "claude-opus-4-6",
            "display_name": "Claude Opus 4.6"
        },
        "workspace": {
            "current_dir": "/home/user/project",
            "project_dir": "/home/user/project"
        },
        "version": "1.0.30",
        "output_style": {
            "name": "text"
        },
        "cost": {
            "total_cost_usd": 0.1234,
            "total_duration_ms": 45000,
            "total_api_duration_ms": 32000,
            "total_lines_added": 120,
            "total_lines_removed": 30
        },
        "context_window": {
            "total_input_tokens": 50000,
            "total_output_tokens": 12000,
            "context_window_size": 200000,
            "used_percentage": 31.0,
            "remaining_percentage": 69.0,
            "current_usage": {
                "input_tokens": 8000,
                "output_tokens": 2000,
                "cache_creation_input_tokens": 1000,
                "cache_read_input_tokens": 5000
            }
        },
        "exceeds_200k_tokens": false,
        "vim": {
            "mode": "normal"
        },
        "agent": {
            "name": "task-agent-1"
        }
    });

    println!("{}", serde_json::to_string_pretty(&sample).unwrap());
}
