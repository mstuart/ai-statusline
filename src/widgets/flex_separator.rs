use super::data::SessionData;
use super::traits::{Widget, WidgetConfig, WidgetOutput};

pub struct FlexSeparatorWidget;

impl Widget for FlexSeparatorWidget {
    fn name(&self) -> &str {
        "flex-separator"
    }

    fn render(&self, _data: &SessionData, config: &WidgetConfig) -> WidgetOutput {
        let fill_char = config
            .metadata
            .get("char")
            .filter(|c| !c.is_empty())
            .cloned()
            .unwrap_or_else(|| " ".to_string());

        // Return a marker; the layout engine expands this to fill available width.
        // display_width of 0 signals the layout engine to calculate the fill.
        WidgetOutput {
            text: fill_char,
            display_width: 0,
            priority: 100,
            visible: true,
            color_hint: None,
        }
    }
}
