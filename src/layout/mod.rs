use unicode_width::UnicodeWidthStr;

use crate::config::Config;
use crate::render::Renderer;
use crate::themes::Theme;
use crate::widgets::{SessionData, WidgetOutput, WidgetRegistry};

pub struct LayoutEngine<'a> {
    config: &'a Config,
    renderer: &'a Renderer,
    theme: Theme,
}

impl<'a> LayoutEngine<'a> {
    pub fn new(config: &'a Config, renderer: &'a Renderer) -> Self {
        let theme = Theme::get(&config.theme);
        Self {
            config,
            renderer,
            theme,
        }
    }

    pub fn render(
        &self,
        data: &SessionData,
        _config: &Config,
        registry: &WidgetRegistry,
    ) -> Vec<String> {
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
                if let Some(output) = registry.render(&wc.widget_type, data, &widget_config)
                    && output.visible
                {
                    widgets.push((output, wc));
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
            let max_display_width = output_lines
                .iter()
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

    /// Resolve the foreground color for a widget using the priority chain:
    /// explicit config color > widget color_hint > theme role > None
    fn resolve_fg_color(
        &self,
        wc: &crate::config::LineWidgetConfig,
        output: &WidgetOutput,
    ) -> Option<String> {
        // 1. Explicit config color
        if let Some(ref color) = wc.color {
            return Some(color.clone());
        }
        // 2. Widget color_hint (dynamic, e.g. context percentage)
        if let Some(ref hint) = output.color_hint {
            return Some(hint.clone());
        }
        // 3. Theme role for this widget type
        if let Some(theme_color) = self.theme.role_for_widget(&wc.widget_type) {
            return Some(theme_color.to_string());
        }
        None
    }

    fn assemble_line(
        &self,
        widgets: &[(WidgetOutput, &crate::config::LineWidgetConfig)],
        max_width: usize,
    ) -> String {
        let config = self.config;
        let separator = &config.default_separator;

        // Check for flex-separator
        let has_flex = widgets
            .iter()
            .any(|(_, wc)| wc.widget_type == "flex-separator");

        if has_flex {
            return self.assemble_line_with_flex(widgets, max_width);
        }

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
            let styled = self.apply_style(&output.text, wc, output);
            parts.push(format!("{padding}{styled}{padding}"));
            total_display_width += output.display_width + UnicodeWidthStr::width(padding) * 2;
        }

        let result = parts.join("");
        format!("{result}{}", self.renderer.reset())
    }

    fn assemble_line_with_flex(
        &self,
        widgets: &[(WidgetOutput, &crate::config::LineWidgetConfig)],
        max_width: usize,
    ) -> String {
        let config = self.config;
        let separator = &config.default_separator;

        // First pass: calculate total width of non-flex widgets
        let mut fixed_width = 0usize;
        for (i, (output, wc)) in widgets.iter().enumerate() {
            if wc.widget_type == "flex-separator" {
                continue;
            }
            let need_separator = i > 0
                && !widgets[i - 1].1.merge_next
                && widgets[i - 1].1.widget_type != "flex-separator";
            if need_separator {
                fixed_width += UnicodeWidthStr::width(separator.as_str());
            }
            let padding = wc.padding.as_deref().unwrap_or(&config.default_padding);
            fixed_width += output.display_width + UnicodeWidthStr::width(padding) * 2;
        }

        let flex_width = max_width.saturating_sub(fixed_width);

        // Second pass: build output
        let mut parts: Vec<String> = Vec::new();
        for (i, (output, wc)) in widgets.iter().enumerate() {
            if wc.widget_type == "flex-separator" {
                // output.text holds the fill character
                let fill_char = &output.text;
                let fill = fill_char.repeat(flex_width);
                let styled = self.apply_style(&fill, wc, output);
                parts.push(styled);
                continue;
            }

            let need_separator = i > 0
                && !widgets[i - 1].1.merge_next
                && widgets[i - 1].1.widget_type != "flex-separator";
            if need_separator {
                parts.push(separator.clone());
            }

            let padding = wc.padding.as_deref().unwrap_or(&config.default_padding);
            let styled = self.apply_style(&output.text, wc, output);
            parts.push(format!("{padding}{styled}{padding}"));
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

        // Check for flex-separator
        let has_flex = widgets
            .iter()
            .any(|(_, wc)| wc.widget_type == "flex-separator");

        // Filter out flex-separator for powerline; compute flex fill
        let non_flex: Vec<&(WidgetOutput, &crate::config::LineWidgetConfig)> = if has_flex {
            widgets
                .iter()
                .filter(|(_, wc)| wc.widget_type != "flex-separator")
                .collect()
        } else {
            widgets.iter().collect()
        };

        let mut parts: Vec<String> = Vec::new();
        let mut total_display_width: usize = 0;

        // Start cap
        if let Some(ref cap) = config.powerline.start_cap {
            let first_bg = non_flex
                .first()
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

        // Find flex index (position in original widgets array)
        let flex_idx = if has_flex {
            widgets
                .iter()
                .position(|(_, wc)| wc.widget_type == "flex-separator")
        } else {
            None
        };

        // Build left part (before flex) and right part (after flex)
        // For powerline with flex, we render left widgets, then fill, then right widgets
        if let Some(fidx) = flex_idx {
            let left_widgets: Vec<&(WidgetOutput, &crate::config::LineWidgetConfig)> = widgets
                [..fidx]
                .iter()
                .filter(|(_, wc)| wc.widget_type != "flex-separator")
                .collect();
            let right_widgets: Vec<&(WidgetOutput, &crate::config::LineWidgetConfig)> = widgets
                [fidx + 1..]
                .iter()
                .filter(|(_, wc)| wc.widget_type != "flex-separator")
                .collect();

            // Render left side
            self.render_powerline_segment(
                &left_widgets,
                &mut parts,
                &mut total_display_width,
                max_width,
                default_bg,
            );

            // End left side with separator to reset
            if let Some(last_left) = left_widgets.last() {
                let last_bg = last_left
                    .1
                    .background_color
                    .as_deref()
                    .unwrap_or(default_bg);
                let last_bg_spec = Renderer::parse_color(last_bg);
                parts.push(format!(
                    "{}{}{}",
                    self.renderer.fg(&last_bg_spec),
                    pl_sep,
                    self.renderer.reset(),
                ));
                total_display_width += UnicodeWidthStr::width(pl_sep.as_str());
            }

            // Calculate right side width
            let mut right_width = 0usize;
            for (i, (output, wc)) in right_widgets.iter().enumerate() {
                if i > 0 {
                    right_width += UnicodeWidthStr::width(pl_sep.as_str());
                }
                let padding = wc.padding.as_deref().unwrap_or(&config.default_padding);
                right_width += output.display_width + UnicodeWidthStr::width(padding) * 2;
            }
            // Add start separator for right side
            if !right_widgets.is_empty() {
                right_width += UnicodeWidthStr::width(pl_sep.as_str());
            }

            // Fill gap
            let fill_width = max_width.saturating_sub(total_display_width + right_width);
            if fill_width > 0 {
                parts.push(" ".repeat(fill_width));
                total_display_width += fill_width;
            }

            // Render right side
            if !right_widgets.is_empty() {
                // Start with separator into first right widget
                let first_bg = right_widgets
                    .first()
                    .and_then(|(_, wc)| wc.background_color.as_deref())
                    .unwrap_or(default_bg);
                let first_bg_spec = Renderer::parse_color(first_bg);
                parts.push(format!(
                    "{}{}{}",
                    self.renderer.fg(&first_bg_spec),
                    "\u{E0B2}", // reverse powerline separator
                    self.renderer.reset(),
                ));
                total_display_width += 1;

                self.render_powerline_segment(
                    &right_widgets,
                    &mut parts,
                    &mut total_display_width,
                    max_width,
                    default_bg,
                );
            }
        } else {
            // No flex — standard powerline assembly
            let all_refs: Vec<&(WidgetOutput, &crate::config::LineWidgetConfig)> =
                non_flex.to_vec();
            self.render_powerline_segment(
                &all_refs,
                &mut parts,
                &mut total_display_width,
                max_width,
                default_bg,
            );
        }

        // End cap
        if let Some(ref cap) = config.powerline.end_cap {
            let last_bg = non_flex
                .last()
                .and_then(|(_, wc)| wc.background_color.as_deref())
                .unwrap_or(default_bg);
            let last_bg_spec = Renderer::parse_color(last_bg);
            parts.push(format!(
                "{}{}{}",
                self.renderer.fg(&last_bg_spec),
                cap,
                self.renderer.reset(),
            ));
        }

        let result = parts.join("");
        format!("{result}{}", self.renderer.reset())
    }

    fn render_powerline_segment(
        &self,
        widgets: &[&(WidgetOutput, &crate::config::LineWidgetConfig)],
        parts: &mut Vec<String>,
        total_display_width: &mut usize,
        max_width: usize,
        default_bg: &str,
    ) {
        let config = self.config;
        let pl_sep = &config.powerline.separator;

        for (i, (output, wc)) in widgets.iter().enumerate() {
            let this_bg = wc.background_color.as_deref().unwrap_or(default_bg);
            let this_bg_spec = Renderer::parse_color(this_bg);

            if i > 0 && !widgets[i - 1].1.merge_next {
                let prev_bg = widgets[i - 1]
                    .1
                    .background_color
                    .as_deref()
                    .unwrap_or(default_bg);
                let prev_bg_spec = Renderer::parse_color(prev_bg);

                let sep_width = UnicodeWidthStr::width(pl_sep.as_str());
                if *total_display_width + sep_width + output.display_width > max_width {
                    break;
                }

                parts.push(format!(
                    "{}{}{}{}",
                    self.renderer.fg(&prev_bg_spec),
                    self.renderer.bg(&this_bg_spec),
                    pl_sep,
                    self.renderer.reset(),
                ));
                *total_display_width += sep_width;
            }

            if *total_display_width + output.display_width > max_width {
                break;
            }

            let padding = wc.padding.as_deref().unwrap_or(&config.default_padding);
            let styled = self.apply_powerline_style(&output.text, wc, &this_bg_spec, output);
            parts.push(styled);

            let padding_width = UnicodeWidthStr::width(padding) * 2;
            *total_display_width += output.display_width + padding_width;
        }
    }

    fn apply_style(
        &self,
        text: &str,
        wc: &crate::config::LineWidgetConfig,
        output: &WidgetOutput,
    ) -> String {
        let config = self.config;
        let mut styled = String::new();

        if let Some(ref bg) = wc.background_color {
            styled.push_str(&self.renderer.bg(&Renderer::parse_color(bg)));
        }

        if let Some(fg) = self.resolve_fg_color(wc, output) {
            styled.push_str(&self.renderer.fg(&Renderer::parse_color(&fg)));
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
        output: &WidgetOutput,
    ) -> String {
        let config = self.config;
        let padding = wc.padding.as_deref().unwrap_or(&config.default_padding);
        let mut styled = String::new();

        // Always set background for powerline segments
        styled.push_str(&self.renderer.bg(bg_spec));

        if let Some(fg) = self.resolve_fg_color(wc, output) {
            styled.push_str(&self.renderer.fg(&Renderer::parse_color(&fg)));
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
