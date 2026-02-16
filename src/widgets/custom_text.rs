use super::traits::{Widget, WidgetConfig, WidgetOutput};
use super::data::SessionData;
use unicode_width::UnicodeWidthStr;

pub struct CustomTextWidget;

impl Widget for CustomTextWidget {
    fn name(&self) -> &str { "custom-text" }

    fn render(&self, _data: &SessionData, config: &WidgetConfig) -> WidgetOutput {
        let text = match config.metadata.get("text") {
            Some(t) if !t.is_empty() => t.clone(),
            _ => return WidgetOutput { text: String::new(), display_width: 0, priority: 30, visible: false },
        };

        let display_width = UnicodeWidthStr::width(text.as_str());
        WidgetOutput { text, display_width, priority: 30, visible: true }
    }
}
