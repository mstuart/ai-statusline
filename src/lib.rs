pub mod config;
pub mod layout;
pub mod render;
pub mod themes;
pub mod tui;
pub mod widgets;

pub use config::Config;
pub use render::Renderer;
pub use widgets::{Widget, WidgetConfig, WidgetOutput, WidgetRegistry};
