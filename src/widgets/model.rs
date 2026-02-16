use super::traits::{Widget, WidgetConfig, WidgetOutput};
use super::data::SessionData;

pub struct ModelWidget;

impl Widget for ModelWidget {
    fn name(&self) -> &str { "model" }

    fn render(&self, data: &SessionData, config: &WidgetConfig) -> WidgetOutput {
        let model = match &data.model {
            Some(m) => m,
            None => return WidgetOutput { text: String::new(), display_width: 0, priority: 90, visible: false },
        };

        let text = if config.raw_value {
            model.id.clone().unwrap_or_default()
        } else {
            model.display_name.clone()
                .or_else(|| model.id.clone())
                .unwrap_or_default()
        };

        let display_width = text.len();
        WidgetOutput { text, display_width, priority: 90, visible: true }
    }
}
