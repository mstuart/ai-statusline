use super::traits::{Widget, WidgetConfig, WidgetOutput};
use super::data::SessionData;

pub struct ContextPercentageWidget;

impl Widget for ContextPercentageWidget {
    fn name(&self) -> &str { "context-percentage" }

    fn render(&self, data: &SessionData, config: &WidgetConfig) -> WidgetOutput {
        let cw = match &data.context_window {
            Some(cw) => cw,
            None => return WidgetOutput { text: String::new(), display_width: 0, priority: 85, visible: false },
        };

        let pct = match cw.used_percentage {
            Some(p) => p,
            None => return WidgetOutput { text: String::new(), display_width: 0, priority: 85, visible: false },
        };

        let display_pct = if config.metadata.get("inverse").map(|v| v == "true").unwrap_or(false) {
            100.0 - pct
        } else {
            pct
        };

        let text = if config.metadata.get("bar").map(|v| v == "true").unwrap_or(false) {
            let filled = ((display_pct / 100.0) * 10.0).round() as usize;
            let filled = filled.min(10);
            let empty = 10 - filled;
            format!(
                "{}{} {}%",
                "▓".repeat(filled),
                "░".repeat(empty),
                display_pct as u64,
            )
        } else {
            format!("{}%", display_pct as u64)
        };

        let display_width = text.len();
        WidgetOutput { text, display_width, priority: 85, visible: true }
    }
}

pub struct ContextLengthWidget;

impl ContextLengthWidget {
    fn format_compact(n: u64) -> String {
        if n >= 1_000_000 {
            format!("{:.1}M", n as f64 / 1_000_000.0)
        } else if n >= 1_000 {
            format!("{}K", n / 1_000)
        } else {
            n.to_string()
        }
    }
}

impl Widget for ContextLengthWidget {
    fn name(&self) -> &str { "context-length" }

    fn render(&self, data: &SessionData, config: &WidgetConfig) -> WidgetOutput {
        let cw = match &data.context_window {
            Some(cw) => cw,
            None => return WidgetOutput { text: String::new(), display_width: 0, priority: 60, visible: false },
        };

        let usage = match &cw.current_usage {
            Some(u) => u,
            None => return WidgetOutput { text: String::new(), display_width: 0, priority: 60, visible: false },
        };

        let total = usage.input_tokens.unwrap_or(0)
            + usage.cache_creation_input_tokens.unwrap_or(0)
            + usage.cache_read_input_tokens.unwrap_or(0);

        let text = if config.raw_value {
            total.to_string()
        } else {
            Self::format_compact(total)
        };

        let display_width = text.len();
        WidgetOutput { text, display_width, priority: 60, visible: true }
    }
}
