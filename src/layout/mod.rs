use unicode_width::UnicodeWidthStr;

use crate::config::Config;
use crate::render::Renderer;
use crate::widgets::{SessionData, WidgetOutput, WidgetRegistry};

pub struct LayoutEngine<'a> {
    config: &'a Config,
    renderer: &'a Renderer,
}

impl<'a> LayoutEngine<'a> {
    pub fn new(config: &'a Config, renderer: &'a Renderer) -> Self {
        Self { config, renderer }
    }

    pub fn render(&self, data: &SessionData, _config: &Config, registry: &WidgetRegistry) -> Vec<String> {
        let config = self.config;
        let term_width = Self::terminal_width(config);
        let mut output_lines = Vec::new();

        for line_config in &config.lines {
            if line_config.is_empty() {
                continue;
            }

            let mut widgets: Vec<(WidgetOutput, &crate::config::LineWidgetConfig)> = Vec::new();
            for wc in line_config {
                let widget_config = Config::to_widget_config(wc);
                if let Some(output) = registry.render(&wc.widget_type, data, &widget_config) {
                    if output.visible {
                        widgets.push((output, wc));
                    }
                }
            }

            if widgets.is_empty() {
                continue;
            }

            let line = if config.powerline.enabled {
                self.assemble_powerline_line(&widgets, term_width)
            } else {
                self.assemble_line(&widgets, term_width)
            };
            output_lines.push(line);
        }

        if config.powerline.enabled && config.powerline.auto_align && output_lines.len() > 1 {
            let max_display_width = output_lines.iter()
                .map(|l| UnicodeWidthStr::width(strip_ansi(l).as_str()))
                .max()
                .unwrap_or(0);

            for line in &mut output_lines {
                let current_width = UnicodeWidthStr::width(strip_ansi(line).as_str());
                if current_width < max_display_width {
                    let pad = max_display_width - current_width;
                    line.push_str(&" ".repeat(pad));
                }
            }
        }

        output_lines
    }

    fn assemble_line(
        &self,
        widgets: &[(WidgetOutput, &crate::config::LineWidgetConfig)],
        max_width: usize,
    ) -> String {
        let config = self.config;
        let separator = &config.default_separator;
        let mut parts: Vec<String> = Vec::new();
        let mut total_display_width = 0;

        for (i, (output, wc)) in widgets.iter().enumerate() {
            let need_separator = i > 0 && !widgets[i - 1].1.merge_next;

            if need_separator {
                let sep_width = UnicodeWidthStr::width(separator.as_str());
                if total_display_width + sep_width + output.display_width > max_width {
                    break;
                }
                parts.push(separator.clone());
                total_display_width += sep_width;
            }

            if total_display_width + output.display_width > max_width {
                break;
            }

            let padding = wc.padding.as_deref().unwrap_or(&config.default_padding);
            let styled = self.apply_style(&output.text, wc);
            parts.push(format!("{padding}{styled}{padding}"));
            total_display_width += output.display_width + UnicodeWidthStr::width(padding) * 2;
        }

        let result = parts.join("");
        format!("{result}{}", self.renderer.reset())
    }

    fn assemble_powerline_line(
        &self,
        widgets: &[(WidgetOutput, &crate::config::LineWidgetConfig)],
        max_width: usize,
    ) -> String {
        let config = self.config;
        let pl_sep = &config.powerline.separator;
        let default_bg = "black";
        let mut parts: Vec<String> = Vec::new();
        let mut total_display_width: usize = 0;

        // Start cap
        if let Some(ref cap) = config.powerline.start_cap {
            let first_bg = widgets.first()
                .and_then(|(_, wc)| wc.background_color.as_deref())
                .unwrap_or(default_bg);
            let bg_spec = Renderer::parse_color(first_bg);
            parts.push(format!(
                "{}{}{}",
                self.renderer.fg(&bg_spec),
                cap,
                self.renderer.reset(),
            ));
            total_display_width += UnicodeWidthStr::width(cap.as_str());
        }

        for (i, (output, wc)) in widgets.iter().enumerate() {
            let this_bg = wc.background_color.as_deref().unwrap_or(default_bg);
            let this_bg_spec = Renderer::parse_color(this_bg);

            // Powerline separator between widgets (not before first)
            if i > 0 && !widgets[i - 1].1.merge_next {
                let prev_bg = widgets[i - 1].1.background_color.as_deref().unwrap_or(default_bg);
                let prev_bg_spec = Renderer::parse_color(prev_bg);

                let sep_width = UnicodeWidthStr::width(pl_sep.as_str());
                if total_display_width + sep_width + output.display_width > max_width {
                    break;
                }

                // Separator: fg = previous bg, bg = current bg
                parts.push(format!(
                    "{}{}{}{}",
                    self.renderer.fg(&prev_bg_spec),
                    self.renderer.bg(&this_bg_spec),
                    pl_sep,
                    self.renderer.reset(),
                ));
                total_display_width += sep_width;
            }

            if total_display_width + output.display_width > max_width {
                break;
            }

            // Widget content with background color
            let padding = wc.padding.as_deref().unwrap_or(&config.default_padding);
            let styled = self.apply_powerline_style(&output.text, wc, &this_bg_spec);
            parts.push(format!("{styled}"));

            let padding_width = UnicodeWidthStr::width(padding) * 2;
            total_display_width += output.display_width + padding_width;
        }

        // End cap
        if let Some(ref cap) = config.powerline.end_cap {
            let last_bg = widgets.last()
                .and_then(|(_, wc)| wc.background_color.as_deref())
                .unwrap_or(default_bg);
            let last_bg_spec = Renderer::parse_color(last_bg);
            parts.push(format!(
                "{}{}{}",
                self.renderer.fg(&last_bg_spec),
                cap,
                self.renderer.reset(),
            ));
            let _ = UnicodeWidthStr::width(cap.as_str());
        }

        let result = parts.join("");
        format!("{result}{}", self.renderer.reset())
    }

    fn apply_style(
        &self,
        text: &str,
        wc: &crate::config::LineWidgetConfig,
    ) -> String {
        let config = self.config;
        let mut styled = String::new();

        if let Some(ref bg) = wc.background_color {
            styled.push_str(&self.renderer.bg(&Renderer::parse_color(bg)));
        }

        if let Some(ref fg) = wc.color {
            styled.push_str(&self.renderer.fg(&Renderer::parse_color(fg)));
        }

        if wc.bold.unwrap_or(config.global_bold) {
            styled.push_str(self.renderer.bold());
        }

        styled.push_str(text);
        styled.push_str(self.renderer.reset());
        styled
    }

    fn apply_powerline_style(
        &self,
        text: &str,
        wc: &crate::config::LineWidgetConfig,
        bg_spec: &crate::render::ColorSpec,
    ) -> String {
        let config = self.config;
        let padding = wc.padding.as_deref().unwrap_or(&config.default_padding);
        let mut styled = String::new();

        // Always set background for powerline segments
        styled.push_str(&self.renderer.bg(bg_spec));

        if let Some(ref fg) = wc.color {
            styled.push_str(&self.renderer.fg(&Renderer::parse_color(fg)));
        }

        if wc.bold.unwrap_or(config.global_bold) {
            styled.push_str(self.renderer.bold());
        }

        styled.push_str(padding);
        styled.push_str(text);
        styled.push_str(padding);
        styled.push_str(self.renderer.reset());
        styled
    }

    fn terminal_width(config: &Config) -> usize {
        let width = crossterm::terminal::size()
            .map(|(w, _)| w as usize)
            .unwrap_or(120);

        match config.flex_mode.as_str() {
            "full" => width,
            "full-minus-40" => width.saturating_sub(40),
            "compact" => 60,
            _ => width.saturating_sub(40),
        }
    }
}

/// Strip ANSI escape sequences from a string for display width calculation.
fn strip_ansi(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut in_escape = false;
    for ch in s.chars() {
        if in_escape {
            if ch.is_ascii_alphabetic() {
                in_escape = false;
            }
            continue;
        }
        if ch == '\x1b' {
            in_escape = true;
            continue;
        }
        // Skip OSC sequences (\x1b]...\x07)
        out.push(ch);
    }
    out
}
