use super::traits::{Widget, WidgetConfig, WidgetOutput};
use super::data::SessionData;
use unicode_width::UnicodeWidthStr;

pub struct SeparatorWidget;

impl Widget for SeparatorWidget {
    fn name(&self) -> &str { "separator" }

    fn render(&self, _data: &SessionData, config: &WidgetConfig) -> WidgetOutput {
        let text = config.metadata.get("char")
            .filter(|c| !c.is_empty())
            .cloned()
            .unwrap_or_else(|| "|".to_string());

        let display_width = UnicodeWidthStr::width(text.as_str());
        WidgetOutput { text, display_width, priority: 100, visible: true }
    }
}
