use super::traits::{Widget, WidgetConfig, WidgetOutput};
use super::data::SessionData;

pub struct CustomTextWidget;

impl Widget for CustomTextWidget {
    fn name(&self) -> &str { "custom-text" }
    fn render(&self, _data: &SessionData, _config: &WidgetConfig) -> WidgetOutput {
        WidgetOutput { text: String::new(), display_width: 0, priority: 50, visible: false }
    }
}
