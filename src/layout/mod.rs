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

    pub fn render(&self, data: &SessionData, config: &Config, registry: &WidgetRegistry) -> Vec<String> {
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

            let line = self.assemble_line(&widgets, term_width, config);
            output_lines.push(line);
        }

        output_lines
    }

    fn assemble_line(
        &self,
        widgets: &[(WidgetOutput, &crate::config::LineWidgetConfig)],
        max_width: usize,
        config: &Config,
    ) -> String {
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
            let styled = self.apply_style(&output.text, wc, config);
            parts.push(format!("{padding}{styled}{padding}"));
            total_display_width += output.display_width + UnicodeWidthStr::width(padding) * 2;
        }

        let result = parts.join("");
        format!("{result}{}", self.renderer.reset())
    }

    fn apply_style(
        &self,
        text: &str,
        wc: &crate::config::LineWidgetConfig,
        config: &Config,
    ) -> String {
        let mut styled = String::new();

        if let Some(ref bg) = wc.background_color {
            styled.push_str(&self.renderer.bg(&crate::render::Renderer::parse_color(bg)));
        }

        if let Some(ref fg) = wc.color {
            styled.push_str(&self.renderer.fg(&crate::render::Renderer::parse_color(fg)));
        }

        if wc.bold.unwrap_or(config.global_bold) {
            styled.push_str(self.renderer.bold());
        }

        styled.push_str(text);
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
