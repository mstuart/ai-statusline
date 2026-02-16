use super::data::SessionData;
use super::traits::{Widget, WidgetConfig, WidgetOutput};

pub struct LinesChangedWidget;

impl Widget for LinesChangedWidget {
    fn name(&self) -> &str {
        "lines-changed"
    }

    fn render(&self, data: &SessionData, config: &WidgetConfig) -> WidgetOutput {
        let added = data
            .cost
            .as_ref()
            .and_then(|c| c.total_lines_added)
            .unwrap_or(0);
        let removed = data
            .cost
            .as_ref()
            .and_then(|c| c.total_lines_removed)
            .unwrap_or(0);

        if added == 0 && removed == 0 {
            return WidgetOutput {
                text: String::new(),
                display_width: 0,
                priority: 40,
                visible: false,
            };
        }

        let text = if config.raw_value {
            format!("+{added}-{removed}")
        } else {
            format!("+{added} -{removed}")
        };

        let display_width = text.len();
        WidgetOutput {
            text,
            display_width,
            priority: 40,
            visible: true,
        }
    }
}
