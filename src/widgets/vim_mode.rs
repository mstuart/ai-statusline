use super::traits::{Widget, WidgetConfig, WidgetOutput};
use super::data::SessionData;

pub struct VimModeWidget;

impl Widget for VimModeWidget {
    fn name(&self) -> &str { "vim-mode" }

    fn render(&self, data: &SessionData, _config: &WidgetConfig) -> WidgetOutput {
        let vim = match &data.vim {
            Some(v) => v,
            None => return WidgetOutput { text: String::new(), display_width: 0, priority: 95, visible: false },
        };

        let text = vim.mode.clone().unwrap_or_else(|| "NORMAL".to_string());
        let display_width = text.len();
        WidgetOutput { text, display_width, priority: 95, visible: true }
    }
}
