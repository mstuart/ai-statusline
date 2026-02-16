use super::traits::{Widget, WidgetConfig, WidgetOutput};
use super::data::SessionData;

pub struct SeparatorWidget;

impl Widget for SeparatorWidget {
    fn name(&self) -> &str { "separator" }
    fn render(&self, _data: &SessionData, _config: &WidgetConfig) -> WidgetOutput {
        WidgetOutput { text: String::new(), display_width: 0, priority: 50, visible: false }
    }
}
