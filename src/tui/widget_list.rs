use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, List, ListItem, Paragraph};

use super::TuiState;

pub fn draw_widget_list(f: &mut ratatui::Frame, state: &TuiState, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(60), Constraint::Percentage(40)])
        .split(area);

    draw_widget_items(f, state, chunks[0]);
    draw_widget_detail(f, state, chunks[1]);
}

fn draw_widget_items(f: &mut ratatui::Frame, state: &TuiState, area: Rect) {
    let line_label = format!(
        "Line {} of {} (Left/Right to switch, a=add, d=delete, j/k=reorder)",
        state.active_line + 1,
        state.config.lines.len(),
    );

    let widgets = state.config.lines.get(state.active_line);
    let items: Vec<ListItem> = match widgets {
        Some(line) => line
            .iter()
            .enumerate()
            .map(|(i, wc)| {
                let selected = i == state.widget_cursor;
                let marker = if selected { ">" } else { " " };
                let color_info = wc
                    .color
                    .as_deref()
                    .map(|c| format!(" [fg:{c}]"))
                    .unwrap_or_default();
                let bg_info = wc
                    .background_color
                    .as_deref()
                    .map(|c| format!(" [bg:{c}]"))
                    .unwrap_or_default();
                let text = format!("{marker} {}{}{}", wc.widget_type, color_info, bg_info);
                let style = if selected {
                    Style::default()
                        .fg(Color::Cyan)
                        .add_modifier(Modifier::BOLD)
                } else {
                    Style::default().fg(Color::White)
                };
                ListItem::new(Line::from(Span::styled(text, style)))
            })
            .collect(),
        None => vec![ListItem::new(Line::from("  (no widgets)"))],
    };

    let list = List::new(items).block(Block::default().borders(Borders::ALL).title(line_label));
    f.render_widget(list, area);
}

fn draw_widget_detail(f: &mut ratatui::Frame, state: &TuiState, area: Rect) {
    let widget = state
        .config
        .lines
        .get(state.active_line)
        .and_then(|line| line.get(state.widget_cursor));

    let text: Vec<Line> = match widget {
        Some(wc) => {
            let mut lines = vec![
                Line::from(Span::styled(
                    format!("  Type: {}", wc.widget_type),
                    Style::default().fg(Color::White),
                )),
                Line::from(Span::styled(
                    format!("  Color: {}", wc.color.as_deref().unwrap_or("(theme)")),
                    Style::default().fg(Color::White),
                )),
                Line::from(Span::styled(
                    format!(
                        "  Background: {}",
                        wc.background_color.as_deref().unwrap_or("(none)")
                    ),
                    Style::default().fg(Color::White),
                )),
                Line::from(Span::styled(
                    format!(
                        "  Bold: {}",
                        wc.bold
                            .map(|b| if b { "yes" } else { "no" })
                            .unwrap_or("(default)")
                    ),
                    Style::default().fg(Color::White),
                )),
                Line::from(Span::styled(
                    format!("  Raw value: {}", if wc.raw_value { "yes" } else { "no" }),
                    Style::default().fg(Color::White),
                )),
                Line::from(Span::styled(
                    format!("  Merge next: {}", if wc.merge_next { "yes" } else { "no" }),
                    Style::default().fg(Color::White),
                )),
            ];
            if !wc.metadata.is_empty() {
                lines.push(Line::from(Span::styled(
                    "  Metadata:".to_string(),
                    Style::default().fg(Color::DarkGray),
                )));
                for (k, v) in &wc.metadata {
                    lines.push(Line::from(Span::styled(
                        format!("    {k}: {v}"),
                        Style::default().fg(Color::DarkGray),
                    )));
                }
            }
            lines
        }
        None => vec![Line::from("  Select a widget")],
    };

    let block = Block::default()
        .borders(Borders::ALL)
        .title("Widget Detail");
    let paragraph = Paragraph::new(text).block(block);
    f.render_widget(paragraph, area);
}
