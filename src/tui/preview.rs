use ratatui::layout::Rect;
use ratatui::style::{Color, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph};

use crate::layout::LayoutEngine;
use crate::render::Renderer;
use crate::widgets::data::*;
use crate::widgets::{SessionData, WidgetRegistry};

use super::TuiState;

fn mock_session() -> SessionData {
    SessionData {
        cwd: Some("/Users/demo/project".into()),
        session_id: Some("abc12345-def6-7890".into()),
        transcript_path: None,
        model: Some(Model {
            id: Some("claude-opus-4-6".into()),
            display_name: Some("Opus".into()),
        }),
        workspace: Some(Workspace {
            current_dir: Some("/Users/demo/project".into()),
            project_dir: Some("/Users/demo/project".into()),
        }),
        version: Some("2.1.31".into()),
        output_style: Some(OutputStyle {
            name: Some("default".into()),
        }),
        cost: Some(Cost {
            total_cost_usd: Some(0.42),
            total_duration_ms: Some(345000),
            total_api_duration_ms: Some(156000),
            total_lines_added: Some(234),
            total_lines_removed: Some(56),
        }),
        context_window: Some(ContextWindow {
            total_input_tokens: Some(50000),
            total_output_tokens: Some(12000),
            context_window_size: Some(200000),
            used_percentage: Some(65.0),
            remaining_percentage: Some(35.0),
            current_usage: Some(CurrentUsage {
                input_tokens: Some(25000),
                output_tokens: Some(8000),
                cache_creation_input_tokens: Some(10000),
                cache_read_input_tokens: Some(5000),
            }),
        }),
        exceeds_200k_tokens: Some(false),
        vim: None,
        agent: None,
    }
}

pub fn draw_preview(f: &mut ratatui::Frame, state: &TuiState, area: Rect) {
    let data = mock_session();
    let renderer = Renderer::detect("none");
    let registry = WidgetRegistry::new();

    // Use a modified config with full flex mode for preview
    let mut preview_config = state.config.clone();
    preview_config.flex_mode = "compact".to_string();

    let engine = LayoutEngine::new(&preview_config, &renderer);
    let rendered = engine.render(&data, &preview_config, &registry);

    let mut lines: Vec<Line> = vec![
        Line::from(Span::styled(
            "  Live Preview (mock data)",
            Style::default().fg(Color::DarkGray),
        )),
        Line::from(""),
    ];

    if rendered.is_empty() {
        lines.push(Line::from(Span::styled(
            "  (no visible output — add widgets or check config)",
            Style::default().fg(Color::Yellow),
        )));
    } else {
        for (i, line) in rendered.iter().enumerate() {
            lines.push(Line::from(Span::styled(
                format!("  Line {}: {}", i + 1, line),
                Style::default().fg(Color::White),
            )));
        }
    }

    lines.push(Line::from(""));
    lines.push(Line::from(Span::styled(
        format!(
            "  Theme: {} | Powerline: {} | Flex: {}",
            state.config.theme,
            if state.config.powerline.enabled {
                "ON"
            } else {
                "OFF"
            },
            state.config.flex_mode,
        ),
        Style::default().fg(Color::DarkGray),
    )));

    let block = Block::default().borders(Borders::ALL).title("Preview");
    let paragraph = Paragraph::new(lines).block(block);
    f.render_widget(paragraph, area);
}
