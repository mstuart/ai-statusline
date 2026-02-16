use claudeline::config::Config;
use claudeline::layout::LayoutEngine;
use claudeline::render::Renderer;
use claudeline::widgets::{SessionData, WidgetRegistry};

fn render_json(json: &str) -> Vec<String> {
    let data: SessionData = serde_json::from_str(json).expect("Failed to parse JSON");
    let config = Config::default();
    let renderer = Renderer::detect("none");
    let registry = WidgetRegistry::new();
    let engine = LayoutEngine::new(&config, &renderer);
    engine.render(&data, &config, &registry)
}

#[test]
fn parse_full_claude_json_into_session_data() {
    let json = r#"{
        "cwd": "/Users/test/project",
        "session_id": "abc12345-def6-7890-ghij-klmn12345678",
        "transcript_path": "/tmp/transcript.jsonl",
        "model": {
            "id": "claude-opus-4-6",
            "display_name": "Opus"
        },
        "workspace": {
            "current_dir": "/Users/test/project",
            "project_dir": "/Users/test/project"
        },
        "version": "2.1.31",
        "output_style": { "name": "default" },
        "cost": {
            "total_cost_usd": 0.0842,
            "total_duration_ms": 345000,
            "total_api_duration_ms": 156000,
            "total_lines_added": 156,
            "total_lines_removed": 23
        },
        "context_window": {
            "total_input_tokens": 15234,
            "total_output_tokens": 4521,
            "context_window_size": 200000,
            "used_percentage": 42.5,
            "remaining_percentage": 57.5,
            "current_usage": {
                "input_tokens": 8500,
                "output_tokens": 1200,
                "cache_creation_input_tokens": 5000,
                "cache_read_input_tokens": 2000
            }
        },
        "exceeds_200k_tokens": false
    }"#;

    let data: SessionData = serde_json::from_str(json).expect("Failed to parse JSON");
    assert_eq!(
        data.model.as_ref().unwrap().display_name.as_deref(),
        Some("Opus")
    );
    assert_eq!(data.version.as_deref(), Some("2.1.31"));
    assert_eq!(data.cost.as_ref().unwrap().total_cost_usd, Some(0.0842));
}

#[test]
fn render_with_default_config_produces_output() {
    let json = r#"{
        "model": { "id": "claude-opus-4-6", "display_name": "Opus" },
        "cost": {
            "total_cost_usd": 0.05,
            "total_duration_ms": 120000
        },
        "context_window": {
            "used_percentage": 30.0,
            "remaining_percentage": 70.0
        }
    }"#;

    let lines = render_json(json);
    assert!(
        !lines.is_empty(),
        "Should produce at least one line of output"
    );
    let combined = lines.join("");
    assert!(combined.contains("Opus"));
}

#[test]
fn render_with_empty_json_does_not_panic() {
    let lines = render_json("{}");
    // Empty JSON means no widgets have data, so no visible output
    // But it should not panic
    assert!(lines.is_empty() || lines.iter().all(|l| l.is_empty() || l.trim().is_empty()));
}

#[test]
fn render_with_minimal_json_just_model() {
    let json = r#"{ "model": { "display_name": "Sonnet" } }"#;
    let lines = render_json(json);
    // model widget should render, others may not
    // Default config has model, context-percentage, session-cost, session-duration
    // Only model will be visible, but layout only renders a line if it has visible widgets
    assert!(!lines.is_empty());
    let combined = lines.join("");
    assert!(combined.contains("Sonnet"));
}

#[test]
fn render_full_session_data() {
    let json = r#"{
        "cwd": "/Users/test/project",
        "session_id": "abc12345-def6-7890-ghij-klmn12345678",
        "model": { "id": "claude-opus-4-6", "display_name": "Opus" },
        "version": "2.1.31",
        "cost": {
            "total_cost_usd": 0.0842,
            "total_duration_ms": 345000,
            "total_api_duration_ms": 156000,
            "total_lines_added": 156,
            "total_lines_removed": 23
        },
        "context_window": {
            "used_percentage": 42.5,
            "remaining_percentage": 57.5,
            "current_usage": {
                "input_tokens": 8500,
                "output_tokens": 1200,
                "cache_creation_input_tokens": 5000,
                "cache_read_input_tokens": 2000
            }
        },
        "exceeds_200k_tokens": false
    }"#;

    let lines = render_json(json);
    assert!(!lines.is_empty());
    let combined = lines.join("");
    // Default config renders: model, context-percentage, session-cost, session-duration
    assert!(combined.contains("Opus"));
    assert!(combined.contains("42%"));
    assert!(combined.contains("$0.08"));
    assert!(combined.contains("5m"));
}

#[test]
fn multiline_config_produces_multiple_lines() {
    let json = r#"{
        "model": { "display_name": "Opus" },
        "version": "2.1.31",
        "cost": {
            "total_cost_usd": 0.05,
            "total_duration_ms": 60000
        },
        "context_window": {
            "used_percentage": 25.0,
            "remaining_percentage": 75.0,
            "current_usage": {
                "input_tokens": 5000,
                "output_tokens": 1000,
                "cache_creation_input_tokens": 2000,
                "cache_read_input_tokens": 1000
            }
        }
    }"#;

    let data: SessionData = serde_json::from_str(json).expect("Failed to parse JSON");

    // Build a two-line config programmatically since lines is Vec<Vec<LineWidgetConfig>>
    use claudeline::config::LineWidgetConfig;
    use std::collections::HashMap;

    let mut config = Config::default();
    config.lines = vec![
        vec![LineWidgetConfig {
            widget_type: "model".into(),
            id: "1".into(),
            color: None,
            background_color: None,
            bold: None,
            raw_value: false,
            padding: None,
            merge_next: false,
            metadata: HashMap::new(),
        }],
        vec![LineWidgetConfig {
            widget_type: "session-cost".into(),
            id: "2".into(),
            color: None,
            background_color: None,
            bold: None,
            raw_value: true,
            padding: None,
            merge_next: false,
            metadata: HashMap::new(),
        }],
    ];

    let renderer = Renderer::detect("none");
    let registry = WidgetRegistry::new();
    let engine = LayoutEngine::new(&config, &renderer);
    let lines = engine.render(&data, &config, &registry);
    assert_eq!(lines.len(), 2, "Should produce two output lines");
}

#[test]
fn json_with_unknown_fields_still_parses() {
    let json = r#"{
        "model": { "display_name": "Opus" },
        "unknown_field": "should be ignored",
        "another_unknown": 42
    }"#;
    // SessionData derives Deserialize with default, unknown fields should be ignored
    let data: SessionData = serde_json::from_str(json).expect("Should parse with unknown fields");
    assert_eq!(
        data.model.as_ref().unwrap().display_name.as_deref(),
        Some("Opus")
    );
}

#[test]
fn json_with_null_fields_parses() {
    let json = r#"{
        "model": null,
        "cost": null,
        "version": null,
        "context_window": null
    }"#;
    let data: SessionData = serde_json::from_str(json).expect("Should parse with null fields");
    assert!(data.model.is_none());
    assert!(data.cost.is_none());
}

#[test]
fn renderer_none_produces_no_ansi() {
    let json = r#"{
        "model": { "display_name": "Opus" },
        "cost": { "total_cost_usd": 0.05, "total_duration_ms": 60000 },
        "context_window": { "used_percentage": 25.0, "remaining_percentage": 75.0 }
    }"#;

    let lines = render_json(json);
    for line in &lines {
        assert!(
            !line.contains("\x1b["),
            "No ANSI escape codes with color_level=none"
        );
    }
}

#[test]
fn widget_registry_has_all_expected_widgets() {
    let registry = WidgetRegistry::new();
    let data = SessionData::default();
    let config = claudeline::widgets::WidgetConfig {
        widget_type: String::new(),
        id: "test".into(),
        color: None,
        background_color: None,
        bold: None,
        raw_value: false,
        padding: None,
        merge_next: false,
        metadata: std::collections::HashMap::new(),
    };

    let expected = [
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

    for name in &expected {
        assert!(
            registry.render(name, &data, &config).is_some(),
            "Widget '{}' should be registered in the registry",
            name
        );
    }
}
