use super::data::SessionData;
use super::traits::{Widget, WidgetConfig, WidgetOutput};

pub struct ExceedsTokensWidget;

impl Widget for ExceedsTokensWidget {
    fn name(&self) -> &str {
        "exceeds-tokens"
    }

    fn render(&self, data: &SessionData, _config: &WidgetConfig) -> WidgetOutput {
        match data.exceeds_200k_tokens {
            Some(true) => {
                let text = "!200K".to_string();
                let display_width = text.len();
                WidgetOutput {
                    text,
                    display_width,
                    priority: 95,
                    visible: true,
                    color_hint: None,
                }
            }
            _ => WidgetOutput {
                text: String::new(),
                display_width: 0,
                priority: 95,
                visible: false,
                color_hint: None,
            },
        }
    }
}
