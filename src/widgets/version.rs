use super::traits::{Widget, WidgetConfig, WidgetOutput};
use super::data::SessionData;

pub struct VersionWidget;

impl Widget for VersionWidget {
    fn name(&self) -> &str { "version" }

    fn render(&self, data: &SessionData, _config: &WidgetConfig) -> WidgetOutput {
        let ver = match &data.version {
            Some(v) => v,
            None => return WidgetOutput { text: String::new(), display_width: 0, priority: 25, visible: false },
        };

        let text = if ver.starts_with('v') {
            ver.clone()
        } else {
            format!("v{}", ver)
        };

        let display_width = text.len();
        WidgetOutput { text, display_width, priority: 25, visible: true }
    }
}
