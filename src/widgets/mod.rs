pub mod data;
mod registry;
mod traits;

// Widget implementations
mod agent_name;
mod api_duration;
mod block_timer;
mod burn_rate;
mod context;
mod cost;
mod cost_warning;
mod custom_command;
mod custom_text;
mod cwd;
mod duration;
mod exceeds_tokens;
mod flex_separator;
mod git_branch;
mod git_status;
mod git_worktree;
mod lines_changed;
mod model;
mod model_suggest;
mod output_style;
mod separator;
mod session_id;
mod terminal_width;
mod tokens;
mod version;
mod vim_mode;

pub use data::*;
pub use registry::WidgetRegistry;
pub use traits::{Widget, WidgetConfig, WidgetOutput};
