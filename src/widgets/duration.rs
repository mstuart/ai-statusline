use super::traits::{Widget, WidgetConfig, WidgetOutput};
use super::data::SessionData;

pub struct SessionDurationWidget;

fn format_duration(ms: u64, compact: bool) -> String {
    let total_secs = ms / 1000;
    let hours = total_secs / 3600;
    let mins = (total_secs % 3600) / 60;
    let secs = total_secs % 60;

    if compact {
        if hours > 0 {
            format!("{}h{}m", hours, mins)
        } else {
            format!("{}m{}s", mins, secs)
        }
    } else if hours > 0 {
        format!("{}h {}m", hours, mins)
    } else {
        format!("{}m {}s", mins, secs)
    }
}

impl Widget for SessionDurationWidget {
    fn name(&self) -> &str { "session-duration" }

    fn render(&self, data: &SessionData, config: &WidgetConfig) -> WidgetOutput {
        let cost = match &data.cost {
            Some(c) => c,
            None => return WidgetOutput { text: String::new(), display_width: 0, priority: 65, visible: false },
        };

        let duration_ms = match cost.total_duration_ms {
            Some(d) => d,
            None => return WidgetOutput { text: String::new(), display_width: 0, priority: 65, visible: false },
        };

        let text = if config.raw_value {
            format_duration(duration_ms, true)
        } else if config.metadata.get("api_ratio").map(|v| v == "true").unwrap_or(false) {
            if let Some(api_ms) = cost.total_api_duration_ms {
                if duration_ms > 0 {
                    let ratio = (api_ms as f64 / duration_ms as f64 * 100.0) as u64;
                    format!("{} (API: {}%)", format_duration(duration_ms, false), ratio)
                } else {
                    format_duration(duration_ms, false)
                }
            } else {
                format_duration(duration_ms, false)
            }
        } else {
            format_duration(duration_ms, false)
        };

        let display_width = text.len();
        WidgetOutput { text, display_width, priority: 65, visible: true }
    }
}
