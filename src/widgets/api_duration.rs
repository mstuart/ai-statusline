use super::data::SessionData;
use super::traits::{Widget, WidgetConfig, WidgetOutput};

pub struct ApiDurationWidget;

impl Widget for ApiDurationWidget {
    fn name(&self) -> &str {
        "api-duration"
    }

    fn render(&self, data: &SessionData, config: &WidgetConfig) -> WidgetOutput {
        let cost = match &data.cost {
            Some(c) => c,
            None => {
                return WidgetOutput {
                    text: String::new(),
                    display_width: 0,
                    priority: 35,
                    visible: false,
                    color_hint: None,
                };
            }
        };

        let total_ms = match cost.total_duration_ms {
            Some(d) if d > 0 => d,
            _ => {
                return WidgetOutput {
                    text: String::new(),
                    display_width: 0,
                    priority: 35,
                    visible: false,
                    color_hint: None,
                };
            }
        };

        let api_ms = match cost.total_api_duration_ms {
            Some(a) => a,
            None => {
                return WidgetOutput {
                    text: String::new(),
                    display_width: 0,
                    priority: 35,
                    visible: false,
                    color_hint: None,
                };
            }
        };

        let pct = (api_ms as f64 / total_ms as f64 * 100.0) as u64;
        let pct_str = format!("{}%", pct);

        let text = if config.raw_value {
            pct_str
        } else {
            format!("API: {}", pct_str)
        };

        let display_width = text.len();
        WidgetOutput {
            text,
            display_width,
            priority: 35,
            visible: true,
            color_hint: None,
        }
    }
}
