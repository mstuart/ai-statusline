use claudeline::widgets::{SessionData, WidgetConfig, WidgetRegistry};
use claudeline::widgets::data::*;
use std::collections::HashMap;

fn mock_session() -> SessionData {
    SessionData {
        cwd: Some("/Users/test/project".into()),
        session_id: Some("abc12345-def6-7890-ghij-klmn12345678".into()),
        transcript_path: Some("/tmp/transcript.jsonl".into()),
        model: Some(Model { id: Some("claude-opus-4-6".into()), display_name: Some("Opus".into()) }),
        workspace: Some(Workspace { current_dir: Some("/Users/test/project".into()), project_dir: Some("/Users/test/project".into()) }),
        version: Some("2.1.31".into()),
        output_style: Some(OutputStyle { name: Some("default".into()) }),
        cost: Some(Cost {
            total_cost_usd: Some(0.0842),
            total_duration_ms: Some(345000),
            total_api_duration_ms: Some(156000),
            total_lines_added: Some(156),
            total_lines_removed: Some(23),
        }),
        context_window: Some(ContextWindow {
            total_input_tokens: Some(15234),
            total_output_tokens: Some(4521),
            context_window_size: Some(200000),
            used_percentage: Some(42.5),
            remaining_percentage: Some(57.5),
            current_usage: Some(CurrentUsage {
                input_tokens: Some(8500),
                output_tokens: Some(1200),
                cache_creation_input_tokens: Some(5000),
                cache_read_input_tokens: Some(2000),
            }),
        }),
        exceeds_200k_tokens: Some(false),
        vim: None,
        agent: None,
    }
}

fn default_config() -> WidgetConfig {
    WidgetConfig {
        widget_type: String::new(),
        id: "test".into(),
        color: None,
        background_color: None,
        bold: None,
        raw_value: false,
        padding: None,
        merge_next: false,
        metadata: HashMap::new(),
    }
}

fn empty_session() -> SessionData {
    SessionData::default()
}

// ─── ModelWidget ───────────────────────────────────────────────

#[test]
fn model_widget_renders_display_name() {
    let registry = WidgetRegistry::new();
    let data = mock_session();
    let config = default_config();
    let output = registry.render("model", &data, &config).unwrap();
    assert!(output.visible);
    assert_eq!(output.text, "Opus");
}

#[test]
fn model_widget_raw_value_renders_model_id() {
    let registry = WidgetRegistry::new();
    let data = mock_session();
    let mut config = default_config();
    config.raw_value = true;
    let output = registry.render("model", &data, &config).unwrap();
    assert!(output.visible);
    assert_eq!(output.text, "claude-opus-4-6");
}

#[test]
fn model_widget_invisible_when_model_is_none() {
    let registry = WidgetRegistry::new();
    let data = empty_session();
    let config = default_config();
    let output = registry.render("model", &data, &config).unwrap();
    assert!(!output.visible);
}

// ─── ContextPercentageWidget ──────────────────────────────────

#[test]
fn context_percentage_renders_percentage() {
    let registry = WidgetRegistry::new();
    let data = mock_session();
    let config = default_config();
    let output = registry.render("context-percentage", &data, &config).unwrap();
    assert!(output.visible);
    assert_eq!(output.text, "42%");
}

#[test]
fn context_percentage_bar_mode() {
    let registry = WidgetRegistry::new();
    let data = mock_session();
    let mut config = default_config();
    config.metadata.insert("bar".into(), "true".into());
    let output = registry.render("context-percentage", &data, &config).unwrap();
    assert!(output.visible);
    // 42.5% -> round(4.25) = 4 filled, 6 empty
    assert!(output.text.contains("42%"));
    assert!(output.text.contains('▓'));
    assert!(output.text.contains('░'));
}

#[test]
fn context_percentage_inverse_mode() {
    let registry = WidgetRegistry::new();
    let data = mock_session();
    let mut config = default_config();
    config.metadata.insert("inverse".into(), "true".into());
    let output = registry.render("context-percentage", &data, &config).unwrap();
    assert!(output.visible);
    // 100 - 42.5 = 57.5, truncated to 57
    assert_eq!(output.text, "57%");
}

#[test]
fn context_percentage_invisible_without_data() {
    let registry = WidgetRegistry::new();
    let data = empty_session();
    let config = default_config();
    let output = registry.render("context-percentage", &data, &config).unwrap();
    assert!(!output.visible);
}

// ─── ContextLengthWidget ──────────────────────────────────────

#[test]
fn context_length_renders_compact() {
    let registry = WidgetRegistry::new();
    let data = mock_session();
    let config = default_config();
    let output = registry.render("context-length", &data, &config).unwrap();
    assert!(output.visible);
    // input=8500 + cache_creation=5000 + cache_read=2000 = 15500 -> "15K"
    assert_eq!(output.text, "15K");
}

#[test]
fn context_length_raw_value() {
    let registry = WidgetRegistry::new();
    let data = mock_session();
    let mut config = default_config();
    config.raw_value = true;
    let output = registry.render("context-length", &data, &config).unwrap();
    assert!(output.visible);
    assert_eq!(output.text, "15500");
}

#[test]
fn context_length_invisible_without_data() {
    let registry = WidgetRegistry::new();
    let data = empty_session();
    let config = default_config();
    let output = registry.render("context-length", &data, &config).unwrap();
    assert!(!output.visible);
}

// ─── TokenInputWidget ─────────────────────────────────────────

#[test]
fn token_input_renders_formatted() {
    let registry = WidgetRegistry::new();
    let data = mock_session();
    let config = default_config();
    let output = registry.render("tokens-input", &data, &config).unwrap();
    assert!(output.visible);
    assert_eq!(output.text, "In: 8,500");
}

#[test]
fn token_input_raw_value_renders_compact() {
    let registry = WidgetRegistry::new();
    let data = mock_session();
    let mut config = default_config();
    config.raw_value = true;
    let output = registry.render("tokens-input", &data, &config).unwrap();
    assert!(output.visible);
    assert_eq!(output.text, "8K");
}

#[test]
fn token_input_invisible_without_data() {
    let registry = WidgetRegistry::new();
    let data = empty_session();
    let config = default_config();
    let output = registry.render("tokens-input", &data, &config).unwrap();
    assert!(!output.visible);
}

// ─── TokenOutputWidget ────────────────────────────────────────

#[test]
fn token_output_renders_formatted() {
    let registry = WidgetRegistry::new();
    let data = mock_session();
    let config = default_config();
    let output = registry.render("tokens-output", &data, &config).unwrap();
    assert!(output.visible);
    assert_eq!(output.text, "Out: 1,200");
}

#[test]
fn token_output_raw_value() {
    let registry = WidgetRegistry::new();
    let data = mock_session();
    let mut config = default_config();
    config.raw_value = true;
    let output = registry.render("tokens-output", &data, &config).unwrap();
    assert!(output.visible);
    assert_eq!(output.text, "1K");
}

// ─── TokenCachedWidget ────────────────────────────────────────

#[test]
fn token_cached_renders_sum() {
    let registry = WidgetRegistry::new();
    let data = mock_session();
    let config = default_config();
    let output = registry.render("tokens-cached", &data, &config).unwrap();
    assert!(output.visible);
    // 5000 + 2000 = 7000
    assert_eq!(output.text, "Cache: 7,000");
}

#[test]
fn token_cached_raw_value() {
    let registry = WidgetRegistry::new();
    let data = mock_session();
    let mut config = default_config();
    config.raw_value = true;
    let output = registry.render("tokens-cached", &data, &config).unwrap();
    assert!(output.visible);
    assert_eq!(output.text, "7K");
}

// ─── TokenTotalWidget ─────────────────────────────────────────

#[test]
fn token_total_renders_all_tokens() {
    let registry = WidgetRegistry::new();
    let data = mock_session();
    let config = default_config();
    let output = registry.render("tokens-total", &data, &config).unwrap();
    assert!(output.visible);
    // 8500 + 1200 + 5000 + 2000 = 16700
    assert_eq!(output.text, "Total: 16,700");
}

#[test]
fn token_total_raw_value() {
    let registry = WidgetRegistry::new();
    let data = mock_session();
    let mut config = default_config();
    config.raw_value = true;
    let output = registry.render("tokens-total", &data, &config).unwrap();
    assert!(output.visible);
    assert_eq!(output.text, "16K");
}

// ─── SessionCostWidget ────────────────────────────────────────

#[test]
fn session_cost_renders_formatted() {
    let registry = WidgetRegistry::new();
    let data = mock_session();
    let config = default_config();
    let output = registry.render("session-cost", &data, &config).unwrap();
    assert!(output.visible);
    assert_eq!(output.text, "$0.08");
}

#[test]
fn session_cost_invisible_without_data() {
    let registry = WidgetRegistry::new();
    let data = empty_session();
    let config = default_config();
    let output = registry.render("session-cost", &data, &config).unwrap();
    assert!(!output.visible);
}

#[test]
fn session_cost_with_burn_rate() {
    let registry = WidgetRegistry::new();
    let data = mock_session();
    let mut config = default_config();
    config.metadata.insert("burn_rate".into(), "true".into());
    let output = registry.render("session-cost", &data, &config).unwrap();
    assert!(output.visible);
    // $0.08 with burn rate: 0.0842 / (345000/3600000) = 0.0842/0.09583... = ~$0.88/hr
    assert!(output.text.contains("$0.08"));
    assert!(output.text.contains("/hr"));
}

// ─── SessionDurationWidget ────────────────────────────────────

#[test]
fn session_duration_renders_formatted() {
    let registry = WidgetRegistry::new();
    let data = mock_session();
    let config = default_config();
    let output = registry.render("session-duration", &data, &config).unwrap();
    assert!(output.visible);
    // 345000ms = 345s = 5m 45s
    assert_eq!(output.text, "5m 45s");
}

#[test]
fn session_duration_raw_value_compact() {
    let registry = WidgetRegistry::new();
    let data = mock_session();
    let mut config = default_config();
    config.raw_value = true;
    let output = registry.render("session-duration", &data, &config).unwrap();
    assert!(output.visible);
    assert_eq!(output.text, "5m45s");
}

#[test]
fn session_duration_invisible_without_data() {
    let registry = WidgetRegistry::new();
    let data = empty_session();
    let config = default_config();
    let output = registry.render("session-duration", &data, &config).unwrap();
    assert!(!output.visible);
}

// ─── BlockTimerWidget ─────────────────────────────────────────

#[test]
fn block_timer_renders_remaining() {
    let registry = WidgetRegistry::new();
    let data = mock_session();
    let config = default_config();
    let output = registry.render("block-timer", &data, &config).unwrap();
    assert!(output.visible);
    // 345000ms elapsed in block. 18_000_000 - 345000 = 17_655_000ms remaining
    // 17_655_000 / 60_000 = 294.25 mins -> 4h54m
    assert!(output.text.contains("Block:"));
    assert!(output.text.contains("left"));
}

#[test]
fn block_timer_bar_mode() {
    let registry = WidgetRegistry::new();
    let data = mock_session();
    let mut config = default_config();
    config.metadata.insert("bar".into(), "true".into());
    let output = registry.render("block-timer", &data, &config).unwrap();
    assert!(output.visible);
    assert!(output.text.contains('▓') || output.text.contains('░'));
}

#[test]
fn block_timer_invisible_without_data() {
    let registry = WidgetRegistry::new();
    let data = empty_session();
    let config = default_config();
    let output = registry.render("block-timer", &data, &config).unwrap();
    assert!(!output.visible);
}

// ─── CwdWidget ────────────────────────────────────────────────

#[test]
fn cwd_renders_basename() {
    let registry = WidgetRegistry::new();
    let data = mock_session();
    let config = default_config();
    let output = registry.render("cwd", &data, &config).unwrap();
    assert!(output.visible);
    assert_eq!(output.text, "project");
}

#[test]
fn cwd_fish_style() {
    let registry = WidgetRegistry::new();
    let mut data = mock_session();
    // Use a path that won't be abbreviated by home dir
    data.workspace = Some(Workspace {
        current_dir: Some("/var/log/myapp".into()),
        project_dir: Some("/var/log/myapp".into()),
    });
    data.cwd = Some("/var/log/myapp".into());
    let mut config = default_config();
    config.metadata.insert("fish_style".into(), "true".into());
    let output = registry.render("cwd", &data, &config).unwrap();
    assert!(output.visible);
    // /var/log/myapp -> /v/l/myapp
    assert_eq!(output.text, "/v/l/myapp");
}

#[test]
fn cwd_full_mode() {
    let registry = WidgetRegistry::new();
    let mut data = mock_session();
    data.workspace = Some(Workspace {
        current_dir: Some("/var/log/myapp".into()),
        project_dir: Some("/var/log/myapp".into()),
    });
    let mut config = default_config();
    config.metadata.insert("full".into(), "true".into());
    let output = registry.render("cwd", &data, &config).unwrap();
    assert!(output.visible);
    assert_eq!(output.text, "/var/log/myapp");
}

#[test]
fn cwd_invisible_without_data() {
    let registry = WidgetRegistry::new();
    let mut data = empty_session();
    data.workspace = None;
    data.cwd = None;
    let config = default_config();
    let output = registry.render("cwd", &data, &config).unwrap();
    assert!(!output.visible);
}

// ─── LinesChangedWidget ──────────────────────────────────────

#[test]
fn lines_changed_renders_diff() {
    let registry = WidgetRegistry::new();
    let data = mock_session();
    let config = default_config();
    let output = registry.render("lines-changed", &data, &config).unwrap();
    assert!(output.visible);
    assert_eq!(output.text, "+156 -23");
}

#[test]
fn lines_changed_raw_value() {
    let registry = WidgetRegistry::new();
    let data = mock_session();
    let mut config = default_config();
    config.raw_value = true;
    let output = registry.render("lines-changed", &data, &config).unwrap();
    assert!(output.visible);
    assert_eq!(output.text, "+156-23");
}

#[test]
fn lines_changed_invisible_when_zero() {
    let registry = WidgetRegistry::new();
    let mut data = mock_session();
    data.cost = Some(Cost {
        total_cost_usd: Some(0.0),
        total_duration_ms: Some(0),
        total_api_duration_ms: Some(0),
        total_lines_added: Some(0),
        total_lines_removed: Some(0),
    });
    let config = default_config();
    let output = registry.render("lines-changed", &data, &config).unwrap();
    assert!(!output.visible);
}

// ─── VersionWidget ────────────────────────────────────────────

#[test]
fn version_renders_with_prefix() {
    let registry = WidgetRegistry::new();
    let data = mock_session();
    let config = default_config();
    let output = registry.render("version", &data, &config).unwrap();
    assert!(output.visible);
    assert_eq!(output.text, "v2.1.31");
}

#[test]
fn version_already_has_v_prefix() {
    let registry = WidgetRegistry::new();
    let mut data = mock_session();
    data.version = Some("v3.0.0".into());
    let config = default_config();
    let output = registry.render("version", &data, &config).unwrap();
    assert!(output.visible);
    assert_eq!(output.text, "v3.0.0");
}

#[test]
fn version_invisible_without_data() {
    let registry = WidgetRegistry::new();
    let data = empty_session();
    let config = default_config();
    let output = registry.render("version", &data, &config).unwrap();
    assert!(!output.visible);
}

// ─── SessionIdWidget ──────────────────────────────────────────

#[test]
fn session_id_renders_short() {
    let registry = WidgetRegistry::new();
    let data = mock_session();
    let config = default_config();
    let output = registry.render("session-id", &data, &config).unwrap();
    assert!(output.visible);
    assert_eq!(output.text, "abc12345");
}

#[test]
fn session_id_invisible_without_data() {
    let registry = WidgetRegistry::new();
    let data = empty_session();
    let config = default_config();
    let output = registry.render("session-id", &data, &config).unwrap();
    assert!(!output.visible);
}

// ─── VimModeWidget ────────────────────────────────────────────

#[test]
fn vim_mode_invisible_without_vim_data() {
    let registry = WidgetRegistry::new();
    let data = mock_session(); // vim: None
    let config = default_config();
    let output = registry.render("vim-mode", &data, &config).unwrap();
    assert!(!output.visible);
}

#[test]
fn vim_mode_visible_with_vim_data() {
    let registry = WidgetRegistry::new();
    let mut data = mock_session();
    data.vim = Some(Vim { mode: Some("INSERT".into()) });
    let config = default_config();
    let output = registry.render("vim-mode", &data, &config).unwrap();
    assert!(output.visible);
    assert_eq!(output.text, "INSERT");
}

#[test]
fn vim_mode_defaults_to_normal() {
    let registry = WidgetRegistry::new();
    let mut data = mock_session();
    data.vim = Some(Vim { mode: None });
    let config = default_config();
    let output = registry.render("vim-mode", &data, &config).unwrap();
    assert!(output.visible);
    assert_eq!(output.text, "NORMAL");
}

// ─── AgentNameWidget ──────────────────────────────────────────

#[test]
fn agent_name_invisible_by_default() {
    let registry = WidgetRegistry::new();
    let data = mock_session(); // agent: None
    let config = default_config();
    let output = registry.render("agent-name", &data, &config).unwrap();
    assert!(!output.visible);
}

#[test]
fn agent_name_visible_with_agent_data() {
    let registry = WidgetRegistry::new();
    let mut data = mock_session();
    data.agent = Some(Agent { name: Some("researcher".into()) });
    let config = default_config();
    let output = registry.render("agent-name", &data, &config).unwrap();
    assert!(output.visible);
    assert_eq!(output.text, "researcher");
}

#[test]
fn agent_name_invisible_with_empty_name() {
    let registry = WidgetRegistry::new();
    let mut data = mock_session();
    data.agent = Some(Agent { name: Some("".into()) });
    let config = default_config();
    let output = registry.render("agent-name", &data, &config).unwrap();
    assert!(!output.visible);
}

// ─── ExceedsTokensWidget ─────────────────────────────────────

#[test]
fn exceeds_tokens_invisible_when_false() {
    let registry = WidgetRegistry::new();
    let data = mock_session(); // exceeds_200k_tokens: Some(false)
    let config = default_config();
    let output = registry.render("exceeds-tokens", &data, &config).unwrap();
    assert!(!output.visible);
}

#[test]
fn exceeds_tokens_visible_when_true() {
    let registry = WidgetRegistry::new();
    let mut data = mock_session();
    data.exceeds_200k_tokens = Some(true);
    let config = default_config();
    let output = registry.render("exceeds-tokens", &data, &config).unwrap();
    assert!(output.visible);
    assert_eq!(output.text, "!200K");
}

#[test]
fn exceeds_tokens_invisible_when_none() {
    let registry = WidgetRegistry::new();
    let mut data = mock_session();
    data.exceeds_200k_tokens = None;
    let config = default_config();
    let output = registry.render("exceeds-tokens", &data, &config).unwrap();
    assert!(!output.visible);
}

// ─── CustomTextWidget ─────────────────────────────────────────

#[test]
fn custom_text_renders_metadata_text() {
    let registry = WidgetRegistry::new();
    let data = mock_session();
    let mut config = default_config();
    config.metadata.insert("text".into(), "Hello World".into());
    let output = registry.render("custom-text", &data, &config).unwrap();
    assert!(output.visible);
    assert_eq!(output.text, "Hello World");
}

#[test]
fn custom_text_invisible_without_text() {
    let registry = WidgetRegistry::new();
    let data = mock_session();
    let config = default_config();
    let output = registry.render("custom-text", &data, &config).unwrap();
    assert!(!output.visible);
}

#[test]
fn custom_text_invisible_with_empty_text() {
    let registry = WidgetRegistry::new();
    let data = mock_session();
    let mut config = default_config();
    config.metadata.insert("text".into(), "".into());
    let output = registry.render("custom-text", &data, &config).unwrap();
    assert!(!output.visible);
}

// ─── SeparatorWidget ──────────────────────────────────────────

#[test]
fn separator_renders_default_pipe() {
    let registry = WidgetRegistry::new();
    let data = mock_session();
    let config = default_config();
    let output = registry.render("separator", &data, &config).unwrap();
    assert!(output.visible);
    assert_eq!(output.text, "|");
}

#[test]
fn separator_renders_custom_char() {
    let registry = WidgetRegistry::new();
    let data = mock_session();
    let mut config = default_config();
    config.metadata.insert("char".into(), "::".into());
    let output = registry.render("separator", &data, &config).unwrap();
    assert!(output.visible);
    assert_eq!(output.text, "::");
}

// ─── TerminalWidthWidget ─────────────────────────────────────

#[test]
fn terminal_width_renders_a_number() {
    let registry = WidgetRegistry::new();
    let data = mock_session();
    let config = default_config();
    let output = registry.render("terminal-width", &data, &config).unwrap();
    assert!(output.visible);
    // Should contain "cols" since raw_value is false
    assert!(output.text.contains("cols"));
}

#[test]
fn terminal_width_raw_value() {
    let registry = WidgetRegistry::new();
    let data = mock_session();
    let mut config = default_config();
    config.raw_value = true;
    let output = registry.render("terminal-width", &data, &config).unwrap();
    assert!(output.visible);
    // Should be just a number
    assert!(output.text.parse::<u16>().is_ok());
}

// ─── OutputStyleWidget ────────────────────────────────────────

#[test]
fn output_style_invisible_when_default() {
    let registry = WidgetRegistry::new();
    let data = mock_session(); // output_style: "default"
    let config = default_config();
    let output = registry.render("output-style", &data, &config).unwrap();
    assert!(!output.visible);
}

#[test]
fn output_style_visible_when_non_default() {
    let registry = WidgetRegistry::new();
    let mut data = mock_session();
    data.output_style = Some(OutputStyle { name: Some("streaming".into()) });
    let config = default_config();
    let output = registry.render("output-style", &data, &config).unwrap();
    assert!(output.visible);
    assert_eq!(output.text, "streaming");
}

// ─── ApiDurationWidget ────────────────────────────────────────

#[test]
fn api_duration_renders_percentage() {
    let registry = WidgetRegistry::new();
    let data = mock_session();
    let config = default_config();
    let output = registry.render("api-duration", &data, &config).unwrap();
    assert!(output.visible);
    // 156000/345000 * 100 = ~45%
    assert_eq!(output.text, "API: 45%");
}

#[test]
fn api_duration_raw_value() {
    let registry = WidgetRegistry::new();
    let data = mock_session();
    let mut config = default_config();
    config.raw_value = true;
    let output = registry.render("api-duration", &data, &config).unwrap();
    assert!(output.visible);
    assert_eq!(output.text, "45%");
}

// ─── All widgets with empty SessionData ───────────────────────

#[test]
fn all_widgets_with_empty_session_no_panic() {
    let registry = WidgetRegistry::new();
    let data = empty_session();
    let config = default_config();

    let widget_names = [
        "model",
        "context-percentage",
        "context-length",
        "tokens-input",
        "tokens-output",
        "tokens-cached",
        "tokens-total",
        "session-cost",
        "session-duration",
        "block-timer",
        "git-branch",
        "git-status",
        "git-worktree",
        "cwd",
        "lines-changed",
        "version",
        "session-id",
        "vim-mode",
        "agent-name",
        "output-style",
        "exceeds-tokens",
        "api-duration",
        "custom-command",
        "custom-text",
        "separator",
        "terminal-width",
    ];

    for name in &widget_names {
        let result = registry.render(name, &data, &config);
        assert!(result.is_some(), "Widget '{}' should be registered", name);
    }
}

#[test]
fn unknown_widget_returns_none() {
    let registry = WidgetRegistry::new();
    let data = mock_session();
    let config = default_config();
    let result = registry.render("nonexistent-widget", &data, &config);
    assert!(result.is_none());
}
