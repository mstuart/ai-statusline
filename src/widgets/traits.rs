use std::collections::HashMap;

use super::data::SessionData;

pub struct WidgetOutput {
    pub text: String,
    pub display_width: usize,
    pub priority: u8,
    pub visible: bool,
}

#[derive(Debug, Clone, Default)]
pub struct WidgetConfig {
    pub widget_type: String,
    pub id: String,
    pub color: Option<String>,
    pub background_color: Option<String>,
    pub bold: Option<bool>,
    pub raw_value: bool,
    pub padding: Option<String>,
    pub merge_next: bool,
    pub metadata: HashMap<String, String>,
}

pub trait Widget: Send + Sync {
    fn name(&self) -> &str;
    fn render(&self, data: &SessionData, config: &WidgetConfig) -> WidgetOutput;
}
