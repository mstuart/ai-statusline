mod data;
mod registry;
mod traits;

// Widget implementations
mod model;
mod context;
mod tokens;
mod cost;
mod duration;
mod block_timer;
mod git_branch;
mod git_status;
mod git_worktree;
mod cwd;
mod lines_changed;
mod version;
mod session_id;
mod vim_mode;
mod agent_name;
mod output_style;
mod exceeds_tokens;
mod api_duration;
mod custom_command;
mod custom_text;
mod separator;
mod terminal_width;

pub use data::*;
pub use registry::WidgetRegistry;
pub use traits::{Widget, WidgetConfig, WidgetOutput};
