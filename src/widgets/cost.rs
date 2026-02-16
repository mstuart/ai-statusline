use super::traits::{Widget, WidgetConfig, WidgetOutput};
use super::data::SessionData;

pub struct SessionCostWidget;

impl Widget for SessionCostWidget {
    fn name(&self) -> &str { "session-cost" }

    fn render(&self, data: &SessionData, config: &WidgetConfig) -> WidgetOutput {
        let cost = match &data.cost {
            Some(c) => c,
            None => return WidgetOutput { text: String::new(), display_width: 0, priority: 70, visible: false },
        };

        let total_usd = match cost.total_cost_usd {
            Some(v) => v,
            None => return WidgetOutput { text: String::new(), display_width: 0, priority: 70, visible: false },
        };

        let cost_str = format!("${:.2}", total_usd);

        let text = if config.raw_value {
            cost_str
        } else if config.metadata.get("burn_rate").map(|v| v == "true").unwrap_or(false) {
            if let Some(duration_ms) = cost.total_duration_ms {
                if duration_ms > 0 {
                    let hours = duration_ms as f64 / 3_600_000.0;
                    let rate = total_usd / hours;
                    format!("{} (${:.2}/hr)", cost_str, rate)
                } else {
                    cost_str
                }
            } else {
                cost_str
            }
        } else {
            cost_str
        };

        let display_width = text.len();
        WidgetOutput { text, display_width, priority: 70, visible: true }
    }
}
