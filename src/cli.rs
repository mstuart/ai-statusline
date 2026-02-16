use std::collections::HashMap;

use chrono::Datelike;
use clap::Subcommand;

use claude_status::config::{Config, LineWidgetConfig, PowerlineConfig};
use claude_status::themes::Theme;

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
    /// Manage Pro license
    License {
        #[command(subcommand)]
        action: LicenseAction,
    },
    /// Show historical cost statistics (Pro)
    Stats {
        /// Time period: daily, weekly, monthly
        #[arg(long, default_value = "weekly")]
        period: String,
    },
}

#[derive(Subcommand)]
pub enum ThemeAction {
    /// List available themes
    List,
    /// Set active theme
    Set { name: String },
}

#[derive(Subcommand)]
pub enum LicenseAction {
    /// Activate a Pro license key
    Activate {
        /// License key (format: CS-PRO-XXXX-XXXX-XXXX-XXXX)
        key: String,
    },
    /// Deactivate (remove) the current license
    Deactivate,
    /// Show current license status
    Status,
}

pub fn handle_command(cmd: Commands) {
    match cmd {
        Commands::Config => {
            if let Err(e) = claude_status::tui::run_tui() {
                eprintln!("TUI error: {e}");
            }
        }
        Commands::Init => cmd_init(),
        Commands::Doctor => cmd_doctor(),
        Commands::Theme { action } => match action {
            ThemeAction::List => cmd_theme_list(),
            ThemeAction::Set { name } => cmd_theme_set(&name),
        },
        Commands::Preset { name } => cmd_preset(&name),
        Commands::DumpSchema => cmd_dump_schema(),
        Commands::License { action } => match action {
            LicenseAction::Activate { key } => cmd_license_activate(&key),
            LicenseAction::Deactivate => cmd_license_deactivate(),
            LicenseAction::Status => cmd_license_status(),
        },
        Commands::Stats { period } => cmd_stats(&period),
    }
}

fn config_path() -> std::path::PathBuf {
    dirs::config_dir()
        .unwrap_or_else(|| std::path::PathBuf::from(".config"))
        .join("claude-status")
        .join("config.toml")
}

fn cmd_init() {
    let path = config_path();
    if let Some(parent) = path.parent()
        && let Err(e) = std::fs::create_dir_all(parent)
    {
        eprintln!("Error creating config directory: {e}");
        return;
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
    println!(r#"      "command": "claude-status""#);
    println!(r#"    }}"#);
    println!(r#"  }}"#);
}

fn cmd_doctor() {
    println!("claude-status doctor");
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
            "  - Config: not found at {} (run `claude-status init` to create)",
            cfg_path.display()
        );
    }

    // License status
    let pro = claude_status::license::is_pro();
    if pro {
        print_check(true, "License: Pro (active)");
    } else {
        println!("  - License: Free (run `claude-status license activate <key>` to upgrade)");
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

fn cmd_license_activate(key: &str) {
    let validator = claude_status::license::LicenseValidator::new();
    match validator.activate(key) {
        Ok(info) => {
            println!("License activated successfully!");
            println!();
            println!("  Tier:     {:?}", info.tier);
            println!("  Status:   {:?}", info.status);
            println!("  Features: {}", info.features.join(", "));
            if let Some(expires) = info.expires {
                println!("  Expires:  {}", expires.format("%Y-%m-%d"));
            }
            println!();
            println!("Pro features are now enabled.");
        }
        Err(e) => {
            eprintln!("License activation failed: {e}");
        }
    }
}

fn cmd_license_deactivate() {
    let validator = claude_status::license::LicenseValidator::new();
    match validator.deactivate() {
        Ok(()) => {
            println!("License deactivated. Pro features are now disabled.");
        }
        Err(e) => {
            eprintln!("Error deactivating license: {e}");
        }
    }
}

fn cmd_license_status() {
    match claude_status::license::check_pro() {
        Some(info) => {
            println!("claude-status Pro");
            println!("=================");
            println!();
            println!("  Status:   {:?}", info.status);
            println!("  Tier:     {:?}", info.tier);
            println!(
                "  Key:      {}...{}",
                &info.key[..11],
                &info.key[info.key.len() - 4..]
            );
            println!("  Features: {}", info.features.join(", "));
            if let Some(expires) = info.expires {
                println!("  Expires:  {}", expires.format("%Y-%m-%d"));
            } else {
                println!("  Expires:  never");
            }
            if let Some(validated) = info.last_validated {
                println!("  Validated: {}", validated.format("%Y-%m-%d %H:%M UTC"));
            }
            println!("  Machine:  {}", info.machine_id);
        }
        None => {
            let storage = claude_status::license::LicenseStorage::new();
            if let Some(key) = storage.load_key() {
                let validator = claude_status::license::LicenseValidator::new();
                let info = validator.validate(&key);
                println!("claude-status Free (license issue)");
                println!("==================================");
                println!();
                println!("  Status:  {:?}", info.status);
                println!(
                    "  Key:     {}...{}",
                    &key[..11.min(key.len())],
                    &key[key.len().saturating_sub(4)..]
                );
                println!();
                println!("Your license key could not be validated.");
                println!("Run `claude-status license activate <key>` with a valid key.");
            } else {
                println!("claude-status Free");
                println!("==================");
                println!();
                println!("No Pro license is active.");
                println!();
                println!("Upgrade to Pro for cost tracking, burn rate analysis,");
                println!("model routing suggestions, and more.");
                println!();
                println!("  Activate: claude-status license activate <key>");
                println!("  Purchase: https://claude-status.dev/pro");
            }
        }
    }
}

fn cmd_stats(period: &str) {
    if !claude_status::license::is_pro() {
        println!("claude-status Stats (Pro feature)");
        println!("=================================");
        println!();
        println!("Historical stats require a Pro license.");
        println!();
        println!("  Activate: claude-status license activate <key>");
        println!("  Purchase: https://claude-status.dev/pro");
        return;
    }

    let tracker = match claude_status::CostTracker::open() {
        Ok(t) => t,
        Err(e) => {
            eprintln!("Error opening cost database: {e}");
            return;
        }
    };

    let now = chrono::Utc::now();
    let today_start = now
        .date_naive()
        .and_hms_opt(0, 0, 0)
        .unwrap()
        .and_utc()
        .timestamp();
    let yesterday_start = today_start - 86400;
    let week_start = today_start
        - (now.weekday().num_days_from_monday() as i64 * 86400);
    let month_start = now
        .date_naive()
        .with_day(1)
        .unwrap()
        .and_hms_opt(0, 0, 0)
        .unwrap()
        .and_utc()
        .timestamp();
    let now_ts = now.timestamp();

    println!("claude-status Stats");
    println!("===================");
    println!();

    // Daily
    let today_cost = tracker.session_cost_range(today_start, now_ts);
    let yesterday_cost = tracker.session_cost_range(yesterday_start, today_start);
    let daily_change = if yesterday_cost > 0.0 {
        let pct = ((today_cost - yesterday_cost) / yesterday_cost) * 100.0;
        if pct >= 0.0 {
            format!(" (+{:.0}% vs yesterday)", pct)
        } else {
            format!(" ({:.0}% vs yesterday)", pct)
        }
    } else {
        String::new()
    };
    println!(
        "  Daily:   ${:.2}{}",
        today_cost, daily_change
    );

    // Weekly
    let weekly_cost = tracker.session_cost_range(week_start, now_ts);
    let weekly_limit = 200.0;
    let weekly_pct = (weekly_cost / weekly_limit) * 100.0;
    println!(
        "  Weekly:  ${:.2} ({:.0}% of ${:.0} limit)",
        weekly_cost, weekly_pct, weekly_limit
    );

    // Monthly
    let monthly_cost = tracker.session_cost_range(month_start, now_ts);
    let days_elapsed = ((now_ts - month_start) as f64 / 86400.0).max(1.0);
    let avg_daily = monthly_cost / days_elapsed;
    println!(
        "  Monthly: ${:.2} (avg ${:.2}/day)",
        monthly_cost, avg_daily
    );

    // Top sessions
    let range_start = match period {
        "daily" => today_start,
        "monthly" => month_start,
        _ => week_start, // default: weekly
    };
    let top = tracker.top_sessions(range_start, now_ts, 5);
    if !top.is_empty() {
        println!();
        println!("  Top costly sessions ({period}):");
        for (i, session) in top.iter().enumerate() {
            let dt = chrono::DateTime::from_timestamp(session.start_time, 0)
                .map(|d| d.format("%b %d, %H:%M").to_string())
                .unwrap_or_else(|| "unknown".into());
            println!(
                "  {}. {} - ${:.2} ({})",
                i + 1,
                dt,
                session.total_cost,
                session.model
            );
        }
    }

    let session_count = tracker.session_count_range(range_start, now_ts);
    println!();
    println!("  Sessions this {period}: {session_count}");
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
