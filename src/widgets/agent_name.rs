use super::traits::{Widget, WidgetConfig, WidgetOutput};
use super::data::SessionData;

pub struct AgentNameWidget;

impl Widget for AgentNameWidget {
    fn name(&self) -> &str { "agent-name" }

    fn render(&self, data: &SessionData, _config: &WidgetConfig) -> WidgetOutput {
        let agent = match &data.agent {
            Some(a) => a,
            None => return WidgetOutput { text: String::new(), display_width: 0, priority: 85, visible: false },
        };

        let text = agent.name.clone().unwrap_or_default();
        if text.is_empty() {
            return WidgetOutput { text: String::new(), display_width: 0, priority: 85, visible: false };
        }

        let display_width = text.len();
        WidgetOutput { text, display_width, priority: 85, visible: true }
    }
}
