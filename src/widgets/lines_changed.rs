use super::traits::{Widget, WidgetConfig, WidgetOutput};
use super::data::SessionData;

pub struct LinesChangedWidget;

impl Widget for LinesChangedWidget {
    fn name(&self) -> &str { "lines-changed" }
    fn render(&self, _data: &SessionData, _config: &WidgetConfig) -> WidgetOutput {
        WidgetOutput { text: String::new(), display_width: 0, priority: 50, visible: false }
    }
}
