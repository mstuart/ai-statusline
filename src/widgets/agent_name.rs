use super::traits::{Widget, WidgetConfig, WidgetOutput};
use super::data::SessionData;

pub struct AgentNameWidget;

impl Widget for AgentNameWidget {
    fn name(&self) -> &str { "agent-name" }
    fn render(&self, _data: &SessionData, _config: &WidgetConfig) -> WidgetOutput {
        WidgetOutput { text: String::new(), display_width: 0, priority: 50, visible: false }
    }
}
