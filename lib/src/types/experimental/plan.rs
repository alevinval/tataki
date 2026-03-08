use crate::types::experimental::plan_entry::PlanEntry;

#[derive(Debug, PartialEq, Eq)]
pub struct Plan {
    entries: Vec<PlanEntry>,
}

impl Plan {
    pub fn new(entries: Vec<PlanEntry>) -> Self {
        Self { entries }
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
