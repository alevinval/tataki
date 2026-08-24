use serde::Deserialize;
use serde::Serialize;

use crate::types::experimental::plan_entry::PlanEntry;

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

    pub fn as_str(&self) -> String {
        let mut out = String::new();
        for entry in self.entries.iter() {
            let line = format!(
                "{:1} {}\n",
                entry.blueprint_id(),
                entry.planned_for().to_rfc3339()
            );
            out.push_str(&line);
        }
        out
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
