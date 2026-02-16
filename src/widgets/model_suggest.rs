use super::data::SessionData;
use super::traits::{Widget, WidgetConfig, WidgetOutput};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Complexity {
    Simple,
    Medium,
    High,
}

pub struct ModelSuggestWidget;

impl ModelSuggestWidget {
    /// Analyze the complexity of current usage based on available session signals.
    fn analyze_complexity(data: &SessionData) -> Complexity {
        // Heuristic 1: Context window usage -- high usage suggests complex tasks
        let context_pct = data
            .context_window
            .as_ref()
            .and_then(|cw| cw.used_percentage)
            .unwrap_or(0.0);

        if context_pct > 60.0 {
            return Complexity::High;
        }

        // Heuristic 2: Token counts -- high output tokens suggest complex generation
        let output_tokens = data
            .context_window
            .as_ref()
            .and_then(|cw| cw.total_output_tokens)
            .unwrap_or(0);

        if output_tokens > 10_000 {
            return Complexity::High;
        }

        if output_tokens > 3_000 || context_pct > 30.0 {
            return Complexity::Medium;
        }

        Complexity::Simple
    }

    /// Determine the model tier from model id string.
    fn model_tier(model_id: &str) -> Option<&'static str> {
        let lower = model_id.to_lowercase();
        if lower.contains("opus") {
            Some("opus")
        } else if lower.contains("sonnet") {
            Some("sonnet")
        } else if lower.contains("haiku") {
            Some("haiku")
        } else {
            None
        }
    }

    /// Suggest a cheaper model if appropriate.
    fn suggest(
        current_tier: &str,
        complexity: Complexity,
        min_savings: f64,
    ) -> Option<(String, f64)> {
        match (current_tier, complexity) {
            ("opus", Complexity::Simple) => {
                let savings = 0.32;
                if savings >= min_savings {
                    Some(("Sonnet".into(), savings))
                } else {
                    None
                }
            }
            ("opus", Complexity::Medium) => {
                let savings = 0.32;
                if savings >= min_savings {
                    Some(("Sonnet".into(), savings))
                } else {
                    None
                }
            }
            ("sonnet", Complexity::Simple) => {
                let savings = 0.09;
                if savings >= min_savings {
                    Some(("Haiku".into(), savings))
                } else {
                    None
                }
            }
            _ => None,
        }
    }
}

impl Widget for ModelSuggestWidget {
    fn name(&self) -> &str {
        "model-suggest"
    }

    fn render(&self, data: &SessionData, config: &WidgetConfig) -> WidgetOutput {
        // Pro-only: gracefully hidden if not Pro
        if !crate::license::is_pro() {
            return WidgetOutput {
                text: String::new(),
                display_width: 0,
                priority: 60,
                visible: false,
                color_hint: None,
            };
        }

        let model_id = match data.model.as_ref().and_then(|m| m.id.as_deref()) {
            Some(id) => id,
            None => {
                return WidgetOutput {
                    text: String::new(),
                    display_width: 0,
                    priority: 60,
                    visible: false,
                    color_hint: None,
                };
            }
        };

        let current_tier = match Self::model_tier(model_id) {
            Some(t) => t,
            None => {
                return WidgetOutput {
                    text: String::new(),
                    display_width: 0,
                    priority: 60,
                    visible: false,
                    color_hint: None,
                };
            }
        };

        let min_savings: f64 = config
            .metadata
            .get("min_savings")
            .and_then(|v| v.parse().ok())
            .unwrap_or(0.10);

        let complexity = Self::analyze_complexity(data);

        let (suggested_model, savings) =
            match Self::suggest(current_tier, complexity, min_savings) {
                Some(s) => s,
                None => {
                    return WidgetOutput {
                        text: String::new(),
                        display_width: 0,
                        priority: 60,
                        visible: false,
                        color_hint: None,
                    };
                }
            };

        let text = if config.raw_value {
            format!("{}:{:.2}", suggested_model, savings)
        } else {
            format!(
                "\u{1F4A1} Try {} -> Save ${:.2}",
                suggested_model, savings
            )
        };

        let display_width = text.len();
        WidgetOutput {
            text,
            display_width,
            priority: 60,
            visible: true,
            color_hint: Some("cyan".into()),
        }
    }
}
