use super::traits::{Widget, WidgetConfig, WidgetOutput};
use super::data::SessionData;

pub struct TokenInputWidget;

impl Widget for TokenInputWidget {
    fn name(&self) -> &str { "tokens-input" }
    fn render(&self, _data: &SessionData, _config: &WidgetConfig) -> WidgetOutput {
        WidgetOutput { text: String::new(), display_width: 0, priority: 50, visible: false }
    }
}

pub struct TokenOutputWidget;

impl Widget for TokenOutputWidget {
    fn name(&self) -> &str { "tokens-output" }
    fn render(&self, _data: &SessionData, _config: &WidgetConfig) -> WidgetOutput {
        WidgetOutput { text: String::new(), display_width: 0, priority: 50, visible: false }
    }
}

pub struct TokenCachedWidget;

impl Widget for TokenCachedWidget {
    fn name(&self) -> &str { "tokens-cached" }
    fn render(&self, _data: &SessionData, _config: &WidgetConfig) -> WidgetOutput {
        WidgetOutput { text: String::new(), display_width: 0, priority: 50, visible: false }
    }
}

pub struct TokenTotalWidget;

impl Widget for TokenTotalWidget {
    fn name(&self) -> &str { "tokens-total" }
    fn render(&self, _data: &SessionData, _config: &WidgetConfig) -> WidgetOutput {
        WidgetOutput { text: String::new(), display_width: 0, priority: 50, visible: false }
    }
}
