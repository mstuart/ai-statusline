use criterion::{Criterion, criterion_group, criterion_main};

use claudeline::config::{Config, LineWidgetConfig, PowerlineConfig};
use claudeline::layout::LayoutEngine;
use claudeline::render::Renderer;
use claudeline::widgets::data::*;
use claudeline::widgets::{SessionData, WidgetRegistry};
use std::collections::HashMap;

fn mock_session() -> SessionData {
    SessionData {
        cwd: Some("/Users/test/project".into()),
        session_id: Some("abc12345-def6-7890-ghij-klmn12345678".into()),
        transcript_path: Some("/tmp/transcript.jsonl".into()),
        model: Some(Model {
            id: Some("claude-opus-4-6".into()),
            display_name: Some("Opus".into()),
        }),
        workspace: Some(Workspace {
            current_dir: Some("/Users/test/project".into()),
            project_dir: Some("/Users/test/project".into()),
        }),
        version: Some("2.1.31".into()),
        output_style: Some(OutputStyle {
            name: Some("default".into()),
        }),
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

fn widget_colored(widget_type: &str, fg: Option<&str>, bg: Option<&str>) -> LineWidgetConfig {
    let mut w = widget(widget_type);
    w.color = fg.map(String::from);
    w.background_color = bg.map(String::from);
    w
}

fn bench_default_render(c: &mut Criterion) {
    let data = mock_session();
    let config = Config::default();
    let renderer = Renderer::detect("none");
    let registry = WidgetRegistry::new();

    c.bench_function("default_render", |b| {
        b.iter(|| {
            let engine = LayoutEngine::new(&config, &renderer);
            engine.render(&data, &config, &registry)
        })
    });
}

fn bench_powerline_render(c: &mut Criterion) {
    let data = mock_session();
    let config = Config {
        lines: vec![vec![
            widget_colored("model", Some("white"), Some("blue")),
            widget_colored("context-percentage", Some("white"), Some("green")),
            widget_colored("session-cost", Some("white"), Some("yellow")),
            widget_colored("session-duration", Some("white"), Some("red")),
        ]],
        powerline: PowerlineConfig {
            enabled: true,
            separator: "\u{E0B0}".into(),
            separator_invert_background: false,
            start_cap: None,
            end_cap: Some("\u{E0B0}".into()),
            auto_align: false,
        },
        ..Config::default()
    };
    let renderer = Renderer::detect("truecolor");
    let registry = WidgetRegistry::new();

    c.bench_function("powerline_render", |b| {
        b.iter(|| {
            let engine = LayoutEngine::new(&config, &renderer);
            engine.render(&data, &config, &registry)
        })
    });
}

fn bench_json_parsing(c: &mut Criterion) {
    let json = r#"{
        "cwd": "/Users/test/project",
        "session_id": "abc12345",
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
        }
    }"#;

    c.bench_function("json_parsing", |b| {
        b.iter(|| {
            let _data: SessionData = serde_json::from_str(json).unwrap();
        })
    });
}

fn bench_single_widget(c: &mut Criterion) {
    let data = mock_session();
    let registry = WidgetRegistry::new();
    let config = claudeline::widgets::WidgetConfig {
        widget_type: "context-percentage".into(),
        id: "bench".into(),
        color: None,
        background_color: None,
        bold: None,
        raw_value: false,
        padding: None,
        merge_next: false,
        metadata: HashMap::new(),
    };

    c.bench_function("single_widget_render", |b| {
        b.iter(|| registry.render("context-percentage", &data, &config))
    });
}

fn bench_multiline_full(c: &mut Criterion) {
    let data = mock_session();
    let config = Config {
        lines: vec![
            vec![
                widget("model"),
                widget("context-percentage"),
                widget("tokens-input"),
                widget("tokens-output"),
                widget("session-cost"),
                widget("session-duration"),
            ],
            vec![widget("cwd"), widget("lines-changed"), widget("version")],
        ],
        ..Config::default()
    };
    let renderer = Renderer::detect("truecolor");
    let registry = WidgetRegistry::new();

    c.bench_function("multiline_full_render", |b| {
        b.iter(|| {
            let engine = LayoutEngine::new(&config, &renderer);
            engine.render(&data, &config, &registry)
        })
    });
}

criterion_group!(
    benches,
    bench_default_render,
    bench_powerline_render,
    bench_json_parsing,
    bench_single_widget,
    bench_multiline_full,
);
criterion_main!(benches);
