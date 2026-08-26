use serde::Deserialize;
use serde::Serialize;

use crate::types::plan_entry::PlanEntry;

#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize)]
pub struct Plan {
    entries: Vec<PlanEntry>,
}

impl Plan {
    pub fn new(entries: Vec<PlanEntry>) -> Self {
        Self { entries }
    }

    pub fn entries(&self) -> &[PlanEntry] {
        &self.entries
    }
}

impl std::fmt::Display for Plan {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for entry in self.entries.iter() {
            writeln!(
                f,
                "{} {}",
                entry.blueprint_id(),
                entry.planned_for().to_rfc3339()
            )?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod test {

    use super::*;
    use crate::test::d;
    use crate::types::Duration;

    #[test]
    fn test_serde_roundtrip() {
        let sut = Plan::new(vec![
            PlanEntry::new(
                "1".to_string(),
                Duration::hours(1),
                d(2026, 10, 23, 9, 0, 0),
            ),
            PlanEntry::new(
                "2".to_string(),
                Duration::hours(2),
                d(2026, 10, 23, 10, 0, 0),
            ),
        ]);
        let json = serde_json::to_string(&sut).unwrap();
        let back: Plan = serde_json::from_str(&json).unwrap();
        assert_eq!(sut, back);
    }
}
