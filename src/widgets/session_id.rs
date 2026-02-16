use super::traits::{Widget, WidgetConfig, WidgetOutput};
use super::data::SessionData;

pub struct SessionIdWidget;

impl Widget for SessionIdWidget {
    fn name(&self) -> &str { "session-id" }

    fn render(&self, data: &SessionData, _config: &WidgetConfig) -> WidgetOutput {
        let sid = match &data.session_id {
            Some(s) => s,
            None => return WidgetOutput { text: String::new(), display_width: 0, priority: 20, visible: false },
        };

        let text: String = sid.chars().take(8).collect();

        let display_width = text.len();
        WidgetOutput { text, display_width, priority: 20, visible: true }
    }
}
