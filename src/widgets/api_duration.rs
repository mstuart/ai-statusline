use super::traits::{Widget, WidgetConfig, WidgetOutput};
use super::data::SessionData;

pub struct ApiDurationWidget;

impl Widget for ApiDurationWidget {
    fn name(&self) -> &str { "api-duration" }
    fn render(&self, _data: &SessionData, _config: &WidgetConfig) -> WidgetOutput {
        WidgetOutput { text: String::new(), display_width: 0, priority: 50, visible: false }
    }
}
