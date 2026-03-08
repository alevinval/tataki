use chrono::DateTime;
use chrono::Local;

use crate::types::Duration;

/// Models an instance of a blueprint.
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct PlanEntry {
    blueprint_id: String,
    planned_for: DateTime<Local>,
    duration: Duration,
}

impl PlanEntry {
    pub const fn new(
        blueprint_id: String,
        duration: Duration,
        planned_for: DateTime<Local>,
    ) -> Self {
        Self {
            blueprint_id,
            planned_for,
            duration,
        }
    }

    pub fn blueprint_id(&self) -> &str {
        &self.blueprint_id
    }

    pub const fn planned_for(&self) -> DateTime<Local> {
        self.planned_for
    }

    pub const fn duration(&self) -> Duration {
        self.duration
    }
}
