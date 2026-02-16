use super::traits::{Widget, WidgetConfig, WidgetOutput};
use super::data::SessionData;

pub struct SessionDurationWidget;

impl Widget for SessionDurationWidget {
    fn name(&self) -> &str { "session-duration" }
    fn render(&self, _data: &SessionData, _config: &WidgetConfig) -> WidgetOutput {
        WidgetOutput { text: String::new(), display_width: 0, priority: 50, visible: false }
    }
}
