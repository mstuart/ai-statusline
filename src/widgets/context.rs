use super::traits::{Widget, WidgetConfig, WidgetOutput};
use super::data::SessionData;

pub struct ContextPercentageWidget;

impl Widget for ContextPercentageWidget {
    fn name(&self) -> &str { "context-percentage" }
    fn render(&self, _data: &SessionData, _config: &WidgetConfig) -> WidgetOutput {
        WidgetOutput { text: String::new(), display_width: 0, priority: 50, visible: false }
    }
}

pub struct ContextLengthWidget;

impl Widget for ContextLengthWidget {
    fn name(&self) -> &str { "context-length" }
    fn render(&self, _data: &SessionData, _config: &WidgetConfig) -> WidgetOutput {
        WidgetOutput { text: String::new(), display_width: 0, priority: 50, visible: false }
    }
}
