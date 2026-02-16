use super::data::SessionData;
use super::traits::{Widget, WidgetConfig, WidgetOutput};
use crate::storage::CostTracker;

use chrono::{Datelike, Utc};

pub struct CostWarningWidget;

impl CostWarningWidget {
    /// Calculate the start of the current week (Monday 00:00 UTC) as Unix timestamp.
    fn week_start() -> i64 {
        let now = Utc::now();
        let days_since_monday = now.weekday().num_days_from_monday() as i64;
        let start_of_today = now
            .date_naive()
            .and_hms_opt(0, 0, 0)
            .unwrap()
            .and_utc()
            .timestamp();
        start_of_today - (days_since_monday * 86400)
    }

    fn calculate(weekly_limit: f64) -> Option<(f64, f64)> {
        let tracker = CostTracker::open().ok()?;
        let since = Self::week_start();
        let spent = tracker.total_cost_since(since);
        let pct = if weekly_limit > 0.0 {
            (spent / weekly_limit) * 100.0
        } else {
            0.0
        };
        Some((spent, pct))
    }
}

impl Widget for CostWarningWidget {
    fn name(&self) -> &str {
        "cost-warning"
    }

    fn render(&self, _data: &SessionData, config: &WidgetConfig) -> WidgetOutput {
        // Pro-only: gracefully hidden if not Pro
        if !crate::license::is_pro() {
            return WidgetOutput {
                text: String::new(),
                display_width: 0,
                priority: 75,
                visible: false,
                color_hint: None,
            };
        }

        let weekly_limit: f64 = config
            .metadata
            .get("weekly_limit")
            .and_then(|v| v.parse().ok())
            .unwrap_or(200.0);

        let warn_threshold: f64 = config
            .metadata
            .get("warn_threshold")
            .and_then(|v| v.parse().ok())
            .unwrap_or(0.7);

        let critical_threshold: f64 = config
            .metadata
            .get("critical_threshold")
            .and_then(|v| v.parse().ok())
            .unwrap_or(0.9);

        let (spent, pct) = match Self::calculate(weekly_limit) {
            Some(v) => v,
            None => {
                return WidgetOutput {
                    text: String::new(),
                    display_width: 0,
                    priority: 75,
                    visible: false,
                    color_hint: None,
                };
            }
        };

        let fraction = pct / 100.0;

        if fraction < warn_threshold {
            // Below warning threshold: don't show anything
            return WidgetOutput {
                text: String::new(),
                display_width: 0,
                priority: 75,
                visible: false,
                color_hint: None,
            };
        }

        let (text, color) = if fraction >= critical_threshold {
            (
                format!(
                    "{} {:.0}% of weekly limit (${:.0}/${:.0})",
                    "\u{1F534}", // red circle
                    pct,
                    spent,
                    weekly_limit
                ),
                "red".to_string(),
            )
        } else {
            (
                format!(
                    "{} {:.0}% of weekly limit (${:.0}/${:.0})",
                    "\u{26A0}\u{FE0F}", // warning sign
                    pct,
                    spent,
                    weekly_limit
                ),
                "yellow".to_string(),
            )
        };

        let display_width = text.len();
        WidgetOutput {
            text,
            display_width,
            priority: 75,
            visible: true,
            color_hint: Some(color),
        }
    }
}
