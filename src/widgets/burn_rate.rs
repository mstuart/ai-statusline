use super::data::SessionData;
use super::traits::{Widget, WidgetConfig, WidgetOutput};
use crate::storage::CostTracker;

use chrono::Utc;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum BurnStatus {
    VeryLow,
    Safe,
    Moderate,
    High,
    Critical,
}

impl BurnStatus {
    fn color_hint(&self) -> Option<String> {
        match self {
            BurnStatus::VeryLow | BurnStatus::Safe => Some("green".into()),
            BurnStatus::Moderate => Some("yellow".into()),
            BurnStatus::High | BurnStatus::Critical => Some("red".into()),
        }
    }
}

pub struct BurnRateWidget;

impl BurnRateWidget {
    fn calculate(window_minutes: u32, weekly_limit: f64) -> Option<(f64, BurnStatus, f64)> {
        let tracker = CostTracker::open().ok()?;
        let now = Utc::now().timestamp();
        let window_secs = window_minutes as i64 * 60;
        let since = now - window_secs;

        let total_cost = tracker.total_cost_since(since);

        if total_cost <= 0.0 {
            return Some((0.0, BurnStatus::VeryLow, f64::INFINITY));
        }

        let hours = window_minutes as f64 / 60.0;
        let rate_per_hour = total_cost / hours;

        // Safe rate = weekly limit / (7 days * 8 work hours)
        let safe_rate = weekly_limit / 56.0;
        let status = if rate_per_hour < safe_rate * 0.5 {
            BurnStatus::VeryLow
        } else if rate_per_hour < safe_rate {
            BurnStatus::Safe
        } else if rate_per_hour < safe_rate * 1.5 {
            BurnStatus::Moderate
        } else if rate_per_hour < safe_rate * 2.0 {
            BurnStatus::High
        } else {
            BurnStatus::Critical
        };

        let hours_until_limit = if rate_per_hour > 0.0 {
            weekly_limit / rate_per_hour
        } else {
            f64::INFINITY
        };

        Some((rate_per_hour, status, hours_until_limit))
    }
}

impl Widget for BurnRateWidget {
    fn name(&self) -> &str {
        "burn-rate"
    }

    fn render(&self, _data: &SessionData, config: &WidgetConfig) -> WidgetOutput {
        // Pro-only: gracefully hidden if not Pro
        if !crate::license::is_pro() {
            return WidgetOutput {
                text: String::new(),
                display_width: 0,
                priority: 65,
                visible: false,
                color_hint: None,
            };
        }

        let window_minutes: u32 = config
            .metadata
            .get("window_minutes")
            .and_then(|v| v.parse().ok())
            .unwrap_or(60);

        let weekly_limit: f64 = config
            .metadata
            .get("weekly_limit")
            .and_then(|v| v.parse().ok())
            .unwrap_or(200.0);

        let (rate, status, hours_left) = match Self::calculate(window_minutes, weekly_limit) {
            Some(v) => v,
            None => {
                return WidgetOutput {
                    text: String::new(),
                    display_width: 0,
                    priority: 65,
                    visible: false,
                    color_hint: None,
                };
            }
        };

        let text = if config.raw_value {
            format!("{:.2}", rate)
        } else if rate < 0.01 {
            "Burn: idle".into()
        } else if hours_left.is_infinite() || hours_left > 168.0 {
            format!("Burn: ${:.2}/hr", rate)
        } else {
            let hours = hours_left as u64;
            let mins = ((hours_left - hours as f64) * 60.0) as u64;
            format!("Burn: ${:.2}/hr -> limit in {}h {}m", rate, hours, mins)
        };

        let display_width = text.len();
        WidgetOutput {
            text,
            display_width,
            priority: 65,
            visible: true,
            color_hint: status.color_hint(),
        }
    }
}
