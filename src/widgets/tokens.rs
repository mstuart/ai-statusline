use super::traits::{Widget, WidgetConfig, WidgetOutput};
use super::data::SessionData;

fn format_tokens(n: u64, compact: bool) -> String {
    if compact {
        if n >= 1_000_000 {
            format!("{:.1}M", n as f64 / 1_000_000.0)
        } else if n >= 1_000 {
            format!("{}K", n / 1_000)
        } else {
            n.to_string()
        }
    } else {
        let s = n.to_string();
        let mut result = String::new();
        for (i, c) in s.chars().rev().enumerate() {
            if i > 0 && i % 3 == 0 {
                result.push(',');
            }
            result.push(c);
        }
        result.chars().rev().collect()
    }
}

pub struct TokenInputWidget;

impl Widget for TokenInputWidget {
    fn name(&self) -> &str { "tokens-input" }

    fn render(&self, data: &SessionData, config: &WidgetConfig) -> WidgetOutput {
        let usage = match data.context_window.as_ref().and_then(|cw| cw.current_usage.as_ref()) {
            Some(u) => u,
            None => return WidgetOutput { text: String::new(), display_width: 0, priority: 55, visible: false },
        };

        let val = usage.input_tokens.unwrap_or(0);
        let text = if config.raw_value {
            format_tokens(val, true)
        } else {
            format!("In: {}", format_tokens(val, false))
        };

        let display_width = text.len();
        WidgetOutput { text, display_width, priority: 55, visible: true }
    }
}

pub struct TokenOutputWidget;

impl Widget for TokenOutputWidget {
    fn name(&self) -> &str { "tokens-output" }

    fn render(&self, data: &SessionData, config: &WidgetConfig) -> WidgetOutput {
        let usage = match data.context_window.as_ref().and_then(|cw| cw.current_usage.as_ref()) {
            Some(u) => u,
            None => return WidgetOutput { text: String::new(), display_width: 0, priority: 53, visible: false },
        };

        let val = usage.output_tokens.unwrap_or(0);
        let text = if config.raw_value {
            format_tokens(val, true)
        } else {
            format!("Out: {}", format_tokens(val, false))
        };

        let display_width = text.len();
        WidgetOutput { text, display_width, priority: 53, visible: true }
    }
}

pub struct TokenCachedWidget;

impl Widget for TokenCachedWidget {
    fn name(&self) -> &str { "tokens-cached" }

    fn render(&self, data: &SessionData, config: &WidgetConfig) -> WidgetOutput {
        let usage = match data.context_window.as_ref().and_then(|cw| cw.current_usage.as_ref()) {
            Some(u) => u,
            None => return WidgetOutput { text: String::new(), display_width: 0, priority: 51, visible: false },
        };

        let val = usage.cache_creation_input_tokens.unwrap_or(0)
            + usage.cache_read_input_tokens.unwrap_or(0);
        let text = if config.raw_value {
            format_tokens(val, true)
        } else {
            format!("Cache: {}", format_tokens(val, false))
        };

        let display_width = text.len();
        WidgetOutput { text, display_width, priority: 51, visible: true }
    }
}

pub struct TokenTotalWidget;

impl Widget for TokenTotalWidget {
    fn name(&self) -> &str { "tokens-total" }

    fn render(&self, data: &SessionData, config: &WidgetConfig) -> WidgetOutput {
        let usage = match data.context_window.as_ref().and_then(|cw| cw.current_usage.as_ref()) {
            Some(u) => u,
            None => return WidgetOutput { text: String::new(), display_width: 0, priority: 50, visible: false },
        };

        let val = usage.input_tokens.unwrap_or(0)
            + usage.output_tokens.unwrap_or(0)
            + usage.cache_creation_input_tokens.unwrap_or(0)
            + usage.cache_read_input_tokens.unwrap_or(0);
        let text = if config.raw_value {
            format_tokens(val, true)
        } else {
            format!("Total: {}", format_tokens(val, false))
        };

        let display_width = text.len();
        WidgetOutput { text, display_width, priority: 50, visible: true }
    }
}
