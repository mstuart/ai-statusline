use super::data::SessionData;
use super::traits::{Widget, WidgetConfig, WidgetOutput};

pub struct OutputStyleWidget;

impl Widget for OutputStyleWidget {
    fn name(&self) -> &str {
        "output-style"
    }

    fn render(&self, data: &SessionData, _config: &WidgetConfig) -> WidgetOutput {
        let style = match &data.output_style {
            Some(s) => s,
            None => {
                return WidgetOutput {
                    text: String::new(),
                    display_width: 0,
                    priority: 30,
                    visible: false,
                    color_hint: None,
                };
            }
        };

        let name = match &style.name {
            Some(n) if n != "default" => n.clone(),
            _ => {
                return WidgetOutput {
                    text: String::new(),
                    display_width: 0,
                    priority: 30,
                    visible: false,
                    color_hint: None,
                };
            }
        };

        let display_width = name.len();
        WidgetOutput {
            text: name,
            display_width,
            priority: 30,
            visible: true,
            color_hint: None,
        }
    }
}
