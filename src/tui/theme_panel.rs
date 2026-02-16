use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, List, ListItem, Paragraph};

use crate::themes::Theme;

use super::TuiState;

pub fn draw_theme_panel(f: &mut ratatui::Frame, state: &TuiState, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(40), Constraint::Percentage(60)])
        .split(area);

    draw_theme_list(f, state, chunks[0]);
    draw_theme_preview(f, state, chunks[1]);
}

fn draw_theme_list(f: &mut ratatui::Frame, state: &TuiState, area: Rect) {
    let themes = Theme::list();
    let items: Vec<ListItem> = themes
        .iter()
        .enumerate()
        .map(|(i, name)| {
            let selected = i == state.theme_cursor;
            let active = *name == state.config.theme.as_str();
            let marker = if selected { ">" } else { " " };
            let active_marker = if active { " *" } else { "" };
            let text = format!("{marker} {name}{active_marker}");
            let style = if selected {
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD)
            } else if active {
                Style::default().fg(Color::Green)
            } else {
                Style::default().fg(Color::White)
            };
            ListItem::new(Line::from(Span::styled(text, style)))
        })
        .collect();

    let list = List::new(items).block(
        Block::default()
            .borders(Borders::ALL)
            .title("Themes (Enter to select)"),
    );
    f.render_widget(list, area);
}

fn draw_theme_preview(f: &mut ratatui::Frame, state: &TuiState, area: Rect) {
    let themes = Theme::list();
    let theme_name = themes.get(state.theme_cursor).unwrap_or(&"default");
    let theme = Theme::get(theme_name);

    let roles = [
        ("model", "Model color"),
        ("context_ok", "Context OK"),
        ("context_warn", "Context Warning"),
        ("context_critical", "Context Critical"),
        ("git_branch", "Git branch"),
        ("git_clean", "Git clean"),
        ("git_dirty", "Git dirty"),
        ("cost", "Cost"),
        ("duration", "Duration"),
        ("separator_fg", "Separator"),
    ];

    let lines: Vec<Line> = roles
        .iter()
        .map(|(role, label)| {
            let color_str = theme.color(role).unwrap_or("(none)");
            let fg_color = parse_preview_color(color_str);
            Line::from(vec![
                Span::styled(format!("  {label}: "), Style::default().fg(Color::White)),
                Span::styled(format!("████ {color_str}"), Style::default().fg(fg_color)),
            ])
        })
        .collect();

    let block = Block::default()
        .borders(Borders::ALL)
        .title(format!("Theme: {theme_name}"));
    let paragraph = Paragraph::new(lines).block(block);
    f.render_widget(paragraph, area);
}

fn parse_preview_color(s: &str) -> Color {
    if s.starts_with('#') && s.len() == 7 {
        let r = u8::from_str_radix(&s[1..3], 16).unwrap_or(255);
        let g = u8::from_str_radix(&s[3..5], 16).unwrap_or(255);
        let b = u8::from_str_radix(&s[5..7], 16).unwrap_or(255);
        Color::Rgb(r, g, b)
    } else {
        match s {
            "black" => Color::Black,
            "red" => Color::Red,
            "green" => Color::Green,
            "yellow" => Color::Yellow,
            "blue" => Color::Blue,
            "magenta" => Color::Magenta,
            "cyan" => Color::Cyan,
            "white" => Color::White,
            "brightBlack" => Color::DarkGray,
            "brightRed" => Color::LightRed,
            "brightGreen" => Color::LightGreen,
            "brightYellow" => Color::LightYellow,
            "brightBlue" => Color::LightBlue,
            "brightMagenta" => Color::LightMagenta,
            "brightCyan" => Color::LightCyan,
            "brightWhite" => Color::White,
            _ => Color::White,
        }
    }
}
