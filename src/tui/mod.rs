mod preview;
mod theme_panel;
mod widget_list;

use std::io::{self, stdout};

use crossterm::event::{self, Event, KeyCode, KeyModifiers};
use crossterm::execute;
use crossterm::terminal::{
    EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode,
};
use ratatui::Terminal;
use ratatui::backend::CrosstermBackend;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph, Tabs};

use crate::config::{Config, LineWidgetConfig};
use crate::themes::Theme;

use preview::draw_preview;
use theme_panel::draw_theme_panel;
use widget_list::draw_widget_list;

#[derive(Clone, Copy, PartialEq)]
enum Tab {
    Widgets,
    Theme,
    Powerline,
    Layout,
    Preview,
}

impl Tab {
    fn index(self) -> usize {
        match self {
            Tab::Widgets => 0,
            Tab::Theme => 1,
            Tab::Powerline => 2,
            Tab::Layout => 3,
            Tab::Preview => 4,
        }
    }

    fn from_index(i: usize) -> Self {
        match i {
            0 => Tab::Widgets,
            1 => Tab::Theme,
            2 => Tab::Powerline,
            3 => Tab::Layout,
            4 => Tab::Preview,
            _ => Tab::Widgets,
        }
    }

    fn count() -> usize {
        5
    }
}

pub struct TuiState {
    config: Config,
    active_tab: Tab,
    // Widget tab state
    widget_cursor: usize,
    active_line: usize,
    // Theme tab state
    theme_cursor: usize,
    // Powerline tab state
    powerline_cursor: usize,
    // Layout tab state
    layout_cursor: usize,
    // Dirty flag
    modified: bool,
}

impl TuiState {
    fn new(config: Config) -> Self {
        Self {
            config,
            active_tab: Tab::Widgets,
            widget_cursor: 0,
            active_line: 0,
            theme_cursor: 0,
            powerline_cursor: 0,
            layout_cursor: 0,
            modified: false,
        }
    }
}

pub fn run_tui() -> io::Result<()> {
    let config = Config::load(None);
    let mut state = TuiState::new(config);

    enable_raw_mode()?;
    let mut stdout = stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let result = run_loop(&mut terminal, &mut state);

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    result
}

fn run_loop<B: ratatui::backend::Backend>(
    terminal: &mut Terminal<B>,
    state: &mut TuiState,
) -> io::Result<()> {
    loop {
        terminal.draw(|f| draw_ui(f, state))?;

        if event::poll(std::time::Duration::from_millis(100))?
            && let Event::Key(key) = event::read()?
        {
            match key.code {
                KeyCode::Char('q') => {
                    return Ok(());
                }
                KeyCode::Char('s') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                    save_config(&state.config);
                    state.modified = false;
                }
                KeyCode::Tab => {
                    let next = (state.active_tab.index() + 1) % Tab::count();
                    state.active_tab = Tab::from_index(next);
                }
                KeyCode::BackTab => {
                    let prev = if state.active_tab.index() == 0 {
                        Tab::count() - 1
                    } else {
                        state.active_tab.index() - 1
                    };
                    state.active_tab = Tab::from_index(prev);
                }
                _ => handle_tab_input(state, key.code),
            }
        }
    }
}

fn handle_tab_input(state: &mut TuiState, key: KeyCode) {
    match state.active_tab {
        Tab::Widgets => handle_widgets_input(state, key),
        Tab::Theme => handle_theme_input(state, key),
        Tab::Powerline => handle_powerline_input(state, key),
        Tab::Layout => handle_layout_input(state, key),
        Tab::Preview => {}
    }
}

fn handle_widgets_input(state: &mut TuiState, key: KeyCode) {
    let line_count = state
        .config
        .lines
        .get(state.active_line)
        .map(|l| l.len())
        .unwrap_or(0);
    match key {
        KeyCode::Up => {
            if state.widget_cursor > 0 {
                state.widget_cursor -= 1;
            }
        }
        KeyCode::Down => {
            if line_count > 0 && state.widget_cursor < line_count - 1 {
                state.widget_cursor += 1;
            }
        }
        KeyCode::Left => {
            if state.active_line > 0 {
                state.active_line -= 1;
                state.widget_cursor = 0;
            }
        }
        KeyCode::Right => {
            if state.active_line < state.config.lines.len().saturating_sub(1) {
                state.active_line += 1;
                state.widget_cursor = 0;
            }
        }
        KeyCode::Char('a') => {
            // Add a widget
            let available = available_widget_types();
            if let Some(line) = state.config.lines.get_mut(state.active_line) {
                let next_type = available
                    .iter()
                    .find(|t| !line.iter().any(|w| w.widget_type == **t))
                    .unwrap_or(&"custom-text");
                line.push(default_widget(next_type));
                state.modified = true;
            }
        }
        KeyCode::Char('d') | KeyCode::Delete => {
            // Remove widget at cursor
            if let Some(line) = state.config.lines.get_mut(state.active_line)
                && !line.is_empty()
                && state.widget_cursor < line.len()
            {
                line.remove(state.widget_cursor);
                if state.widget_cursor >= line.len() && !line.is_empty() {
                    state.widget_cursor = line.len() - 1;
                }
                state.modified = true;
            }
        }
        KeyCode::Char('k') => {
            // Move widget up
            if let Some(line) = state.config.lines.get_mut(state.active_line)
                && state.widget_cursor > 0
            {
                line.swap(state.widget_cursor, state.widget_cursor - 1);
                state.widget_cursor -= 1;
                state.modified = true;
            }
        }
        KeyCode::Char('j') => {
            // Move widget down
            if let Some(line) = state.config.lines.get_mut(state.active_line)
                && state.widget_cursor + 1 < line.len()
            {
                line.swap(state.widget_cursor, state.widget_cursor + 1);
                state.widget_cursor += 1;
                state.modified = true;
            }
        }
        _ => {}
    }
}

fn handle_theme_input(state: &mut TuiState, key: KeyCode) {
    let themes = Theme::list();
    match key {
        KeyCode::Up => {
            if state.theme_cursor > 0 {
                state.theme_cursor -= 1;
            }
        }
        KeyCode::Down => {
            if state.theme_cursor < themes.len() - 1 {
                state.theme_cursor += 1;
            }
        }
        KeyCode::Enter => {
            if let Some(name) = themes.get(state.theme_cursor) {
                state.config.theme = name.to_string();
                state.modified = true;
            }
        }
        _ => {}
    }
}

fn handle_powerline_input(state: &mut TuiState, key: KeyCode) {
    match key {
        KeyCode::Up => {
            if state.powerline_cursor > 0 {
                state.powerline_cursor -= 1;
            }
        }
        KeyCode::Down => {
            if state.powerline_cursor < 2 {
                state.powerline_cursor += 1;
            }
        }
        KeyCode::Enter | KeyCode::Char(' ') => {
            match state.powerline_cursor {
                0 => {
                    state.config.powerline.enabled = !state.config.powerline.enabled;
                    state.modified = true;
                }
                1 => {
                    // Cycle separator
                    let seps = ["\u{E0B0}", "\u{E0B4}", "\u{E0BC}", "/", "|"];
                    let current = state.config.powerline.separator.as_str();
                    let idx = seps.iter().position(|s| *s == current).unwrap_or(0);
                    state.config.powerline.separator = seps[(idx + 1) % seps.len()].to_string();
                    state.modified = true;
                }
                2 => {
                    state.config.powerline.auto_align = !state.config.powerline.auto_align;
                    state.modified = true;
                }
                _ => {}
            }
        }
        _ => {}
    }
}

fn handle_layout_input(state: &mut TuiState, key: KeyCode) {
    match key {
        KeyCode::Up => {
            if state.layout_cursor > 0 {
                state.layout_cursor -= 1;
            }
        }
        KeyCode::Down => {
            if state.layout_cursor < 2 {
                state.layout_cursor += 1;
            }
        }
        KeyCode::Enter | KeyCode::Char(' ') => {
            match state.layout_cursor {
                0 => {
                    // Add line
                    if state.config.lines.len() < 3 {
                        state.config.lines.push(Vec::new());
                        state.modified = true;
                    }
                }
                1 => {
                    // Remove last line
                    if state.config.lines.len() > 1 {
                        state.config.lines.pop();
                        if state.active_line >= state.config.lines.len() {
                            state.active_line = state.config.lines.len() - 1;
                        }
                        state.modified = true;
                    }
                }
                2 => {
                    // Cycle flex mode
                    let modes = ["full-minus-40", "full", "compact"];
                    let idx = modes
                        .iter()
                        .position(|m| *m == state.config.flex_mode.as_str())
                        .unwrap_or(0);
                    state.config.flex_mode = modes[(idx + 1) % modes.len()].to_string();
                    state.modified = true;
                }
                _ => {}
            }
        }
        _ => {}
    }
}

fn draw_ui(f: &mut ratatui::Frame, state: &TuiState) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Tabs
            Constraint::Min(1),    // Content
            Constraint::Length(1), // Status bar
        ])
        .split(f.area());

    draw_tabs(f, state, chunks[0]);

    match state.active_tab {
        Tab::Widgets => draw_widget_list(f, state, chunks[1]),
        Tab::Theme => draw_theme_panel(f, state, chunks[1]),
        Tab::Powerline => draw_powerline_panel(f, state, chunks[1]),
        Tab::Layout => draw_layout_panel(f, state, chunks[1]),
        Tab::Preview => draw_preview(f, state, chunks[1]),
    }

    draw_status_bar(f, state, chunks[2]);
}

fn draw_tabs(f: &mut ratatui::Frame, state: &TuiState, area: Rect) {
    let titles: Vec<Line> = ["Widgets", "Theme", "Powerline", "Layout", "Preview"]
        .iter()
        .map(|t| Line::from(*t))
        .collect();
    let tabs = Tabs::new(titles)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("claudeline config"),
        )
        .select(state.active_tab.index())
        .style(Style::default().fg(Color::White))
        .highlight_style(
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        );
    f.render_widget(tabs, area);
}

fn draw_powerline_panel(f: &mut ratatui::Frame, state: &TuiState, area: Rect) {
    let pl = &state.config.powerline;
    let items = [
        format!(
            "  {} Enabled: {}",
            if state.powerline_cursor == 0 {
                ">"
            } else {
                " "
            },
            if pl.enabled { "ON" } else { "OFF" },
        ),
        format!(
            "  {} Separator: \"{}\"",
            if state.powerline_cursor == 1 {
                ">"
            } else {
                " "
            },
            pl.separator,
        ),
        format!(
            "  {} Auto-align: {}",
            if state.powerline_cursor == 2 {
                ">"
            } else {
                " "
            },
            if pl.auto_align { "ON" } else { "OFF" },
        ),
    ];

    let text: Vec<Line> = items
        .iter()
        .enumerate()
        .map(|(i, item)| {
            let style = if i == state.powerline_cursor {
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::White)
            };
            Line::from(Span::styled(item.clone(), style))
        })
        .collect();

    let block = Block::default()
        .borders(Borders::ALL)
        .title("Powerline Settings (Enter to toggle/cycle)");
    let paragraph = Paragraph::new(text).block(block);
    f.render_widget(paragraph, area);
}

fn draw_layout_panel(f: &mut ratatui::Frame, state: &TuiState, area: Rect) {
    let items = [
        format!(
            "  {} Add line (current: {} line{})",
            if state.layout_cursor == 0 { ">" } else { " " },
            state.config.lines.len(),
            if state.config.lines.len() == 1 {
                ""
            } else {
                "s"
            },
        ),
        format!(
            "  {} Remove last line",
            if state.layout_cursor == 1 { ">" } else { " " },
        ),
        format!(
            "  {} Flex mode: {}",
            if state.layout_cursor == 2 { ">" } else { " " },
            state.config.flex_mode,
        ),
    ];

    let text: Vec<Line> = items
        .iter()
        .enumerate()
        .map(|(i, item)| {
            let style = if i == state.layout_cursor {
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::White)
            };
            Line::from(Span::styled(item.clone(), style))
        })
        .collect();

    let block = Block::default()
        .borders(Borders::ALL)
        .title("Layout Settings (Enter to action)");
    let paragraph = Paragraph::new(text).block(block);
    f.render_widget(paragraph, area);
}

fn draw_status_bar(f: &mut ratatui::Frame, state: &TuiState, area: Rect) {
    let modified = if state.modified { " [modified]" } else { "" };
    let help = format!(
        " Tab/Shift-Tab: switch tabs | arrows: navigate | Enter: select | q: quit | Ctrl-s: save{}",
        modified
    );
    let bar = Paragraph::new(Line::from(Span::styled(
        help,
        Style::default().fg(Color::DarkGray),
    )));
    f.render_widget(bar, area);
}

fn save_config(config: &Config) {
    let path = Config::default_path().unwrap_or_else(|| {
        dirs::config_dir()
            .unwrap_or_else(|| std::path::PathBuf::from(".config"))
            .join("claudeline")
            .join("config.toml")
    });

    if let Some(parent) = path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }

    let _ = std::fs::write(&path, config.to_toml());
}

fn available_widget_types() -> Vec<&'static str> {
    vec![
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
        "flex-separator",
        "terminal-width",
    ]
}

fn default_widget(widget_type: &str) -> LineWidgetConfig {
    LineWidgetConfig {
        widget_type: widget_type.to_string(),
        id: String::new(),
        color: None,
        background_color: None,
        bold: None,
        raw_value: false,
        padding: None,
        merge_next: false,
        metadata: std::collections::HashMap::new(),
    }
}
