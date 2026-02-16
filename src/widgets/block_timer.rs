use super::traits::{Widget, WidgetConfig, WidgetOutput};
use super::data::SessionData;

const BLOCK_DURATION_MS: u64 = 18_000_000; // 5 hours

pub struct BlockTimerWidget;

fn format_hm(ms: u64) -> String {
    let total_mins = ms / 60_000;
    let hours = total_mins / 60;
    let mins = total_mins % 60;
    if hours > 0 {
        format!("{}h{}m", hours, mins)
    } else {
        format!("{}m", mins)
    }
}

impl Widget for BlockTimerWidget {
    fn name(&self) -> &str { "block-timer" }

    fn render(&self, data: &SessionData, config: &WidgetConfig) -> WidgetOutput {
        let cost = match &data.cost {
            Some(c) => c,
            None => return WidgetOutput { text: String::new(), display_width: 0, priority: 55, visible: false },
        };

        let duration_ms = match cost.total_duration_ms {
            Some(d) => d,
            None => return WidgetOutput { text: String::new(), display_width: 0, priority: 55, visible: false },
        };

        let block_elapsed = duration_ms % BLOCK_DURATION_MS;
        let block_remaining = BLOCK_DURATION_MS - block_elapsed;
        let remaining_str = format_hm(block_remaining);

        let text = if config.metadata.get("bar").map(|v| v == "true").unwrap_or(false) {
            let bar_width: usize = config.metadata.get("bar_width")
                .and_then(|w| w.parse().ok())
                .unwrap_or(16);
            let fraction = block_elapsed as f64 / BLOCK_DURATION_MS as f64;
            let filled = (fraction * bar_width as f64).round() as usize;
            let filled = filled.min(bar_width);
            let empty = bar_width - filled;
            format!("{}{} {}", "▓".repeat(filled), "░".repeat(empty), remaining_str)
        } else {
            format!("Block: {} left", remaining_str)
        };

        let display_width = text.len();
        WidgetOutput { text, display_width, priority: 55, visible: true }
    }
}
