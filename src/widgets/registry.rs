use std::collections::HashMap;

use super::data::SessionData;
use super::traits::{Widget, WidgetConfig, WidgetOutput};

pub struct WidgetRegistry {
    widgets: HashMap<String, Box<dyn Widget>>,
}

impl Default for WidgetRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl WidgetRegistry {
    pub fn new() -> Self {
        let mut registry = Self {
            widgets: HashMap::new(),
        };
        registry.register_defaults();
        registry
    }

    pub fn register(&mut self, widget: Box<dyn Widget>) {
        self.widgets.insert(widget.name().to_string(), widget);
    }

    pub fn render(
        &self,
        widget_type: &str,
        data: &SessionData,
        config: &WidgetConfig,
    ) -> Option<WidgetOutput> {
        self.widgets
            .get(widget_type)
            .map(|w| w.render(data, config))
    }

    fn register_defaults(&mut self) {
        self.register(Box::new(super::model::ModelWidget));
        self.register(Box::new(super::context::ContextPercentageWidget));
        self.register(Box::new(super::context::ContextLengthWidget));
        self.register(Box::new(super::tokens::TokenInputWidget));
        self.register(Box::new(super::tokens::TokenOutputWidget));
        self.register(Box::new(super::tokens::TokenCachedWidget));
        self.register(Box::new(super::tokens::TokenTotalWidget));
        self.register(Box::new(super::cost::SessionCostWidget));
        self.register(Box::new(super::duration::SessionDurationWidget));
        self.register(Box::new(super::block_timer::BlockTimerWidget));
        self.register(Box::new(super::git_branch::GitBranchWidget));
        self.register(Box::new(super::git_status::GitStatusWidget));
        self.register(Box::new(super::git_worktree::GitWorktreeWidget));
        self.register(Box::new(super::cwd::CwdWidget));
        self.register(Box::new(super::lines_changed::LinesChangedWidget));
        self.register(Box::new(super::version::VersionWidget));
        self.register(Box::new(super::session_id::SessionIdWidget));
        self.register(Box::new(super::vim_mode::VimModeWidget));
        self.register(Box::new(super::agent_name::AgentNameWidget));
        self.register(Box::new(super::output_style::OutputStyleWidget));
        self.register(Box::new(super::exceeds_tokens::ExceedsTokensWidget));
        self.register(Box::new(super::api_duration::ApiDurationWidget));
        self.register(Box::new(super::custom_command::CustomCommandWidget));
        self.register(Box::new(super::custom_text::CustomTextWidget));
        self.register(Box::new(super::separator::SeparatorWidget));
        self.register(Box::new(super::terminal_width::TerminalWidthWidget));
        self.register(Box::new(super::flex_separator::FlexSeparatorWidget));
    }
}
