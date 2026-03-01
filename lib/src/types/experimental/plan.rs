use crate::types::experimental::plan_entry::PlannedEntry;

#[derive(Debug, PartialEq, Eq)]
pub struct Plan {
    entries: Vec<PlannedEntry>,
}

impl Plan {
    pub fn from(entries: Vec<PlannedEntry>) -> Self {
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
