use super::traits::{Widget, WidgetConfig, WidgetOutput};
use super::data::SessionData;

pub struct OutputStyleWidget;

impl Widget for OutputStyleWidget {
    fn name(&self) -> &str { "output-style" }

    fn render(&self, data: &SessionData, _config: &WidgetConfig) -> WidgetOutput {
        let style = match &data.output_style {
            Some(s) => s,
            None => return WidgetOutput { text: String::new(), display_width: 0, priority: 30, visible: false },
        };

        let name = match &style.name {
            Some(n) if n != "default" => n.clone(),
            _ => return WidgetOutput { text: String::new(), display_width: 0, priority: 30, visible: false },
        };

        let display_width = name.len();
        WidgetOutput { text: name, display_width, priority: 30, visible: true }
    }
}
