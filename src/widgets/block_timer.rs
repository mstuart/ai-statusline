use super::traits::{Widget, WidgetConfig, WidgetOutput};
use super::data::SessionData;

pub struct BlockTimerWidget;

impl Widget for BlockTimerWidget {
    fn name(&self) -> &str { "block-timer" }
    fn render(&self, _data: &SessionData, _config: &WidgetConfig) -> WidgetOutput {
        WidgetOutput { text: String::new(), display_width: 0, priority: 50, visible: false }
    }
}
