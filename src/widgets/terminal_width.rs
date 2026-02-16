use super::traits::{Widget, WidgetConfig, WidgetOutput};
use super::data::SessionData;

pub struct TerminalWidthWidget;

impl Widget for TerminalWidthWidget {
    fn name(&self) -> &str { "terminal-width" }

    fn render(&self, _data: &SessionData, config: &WidgetConfig) -> WidgetOutput {
        let cols = crossterm::terminal::size()
            .map(|(w, _)| w)
            .unwrap_or(80);

        let text = if config.raw_value {
            format!("{}", cols)
        } else {
            format!("{} cols", cols)
        };

        let display_width = text.len();
        WidgetOutput { text, display_width, priority: 20, visible: true }
    }
}
