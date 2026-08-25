use chrono::DateTime;
use chrono::Local;
use serde::Deserialize;
use serde::Serialize;

use crate::types::Duration;

/// Models an instance of a blueprint.
#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize)]
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

#[cfg(test)]
mod test {

    use super::*;
    use crate::test::d;

    #[test]
    fn test_serde_roundtrip() {
        let sut = PlanEntry::new(
            "1".to_string(),
            Duration::hours(2),
            d(2026, 10, 23, 9, 0, 0),
        );
        let json = serde_json::to_string(&sut).unwrap();
        let back: PlanEntry = serde_json::from_str(&json).unwrap();
        assert_eq!(sut, back);
    }
}
