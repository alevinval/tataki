use chrono::DateTime;
use chrono::Local;

use crate::types::Blueprint;
use crate::types::Duration;

/// Models a collection of blueprints.
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct BlueprintBook {
    entries: Vec<Blueprint>,
}

impl BlueprintBook {
    pub fn from(entries: Vec<Blueprint>) -> Self {
        Self { entries }
    }

    pub fn entries(&self) -> &[Blueprint] {
        &self.entries
    }
}

impl std::fmt::Display for BlueprintBook {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for bp in &self.entries {
            bp.fmt(f)?;
            f.write_str("\n")?;
        }
        Ok(())
    }
}

/// Models an instance of a blueprint.
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct PlanEntry {
    blueprint_id: String,
    planned_for: DateTime<Local>,
    duration: Duration,
}

impl PlanEntry {
    pub fn new<S: Into<String>>(
        blueprint_id: S,
        duration: Duration,
        planned_for: DateTime<Local>,
    ) -> Self {
        Self {
            blueprint_id: blueprint_id.into(),
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

#[derive(Debug, PartialEq, Eq)]
pub struct Plan {
    entries: Vec<PlanEntry>,
}

impl Plan {
    pub fn from(entries: Vec<PlanEntry>) -> Self {
        Self { entries }
    }

    pub fn as_str(&self) -> String {
        let mut out = String::new();
        for entry in self.entries.iter() {
            let line = format!(
                "{:1} {}\n",
                entry.blueprint_id,
                entry.planned_for.to_rfc3339()
            );
            out.push_str(&line);
        }
        out
    }
}

/// Models an entry in the journal of entries.
pub enum JournalEntry {
    Completed { entry: CompletedEntry },
    Postponed { entry: PostponedEntry },
}

/// Models a journal entry for a task that has been completed.
pub struct CompletedEntry {
    blueprint_id: String,
    journaled_at: DateTime<Local>,
}

/// Models a journal entry for a task that has been postponed.
pub struct PostponedEntry {
    blueprint_id: String,
    journaled_at: DateTime<Local>,
}

pub struct Journal {
    entries: Vec<JournalEntry>,
}

impl Journal {
    pub fn from(entries: Vec<JournalEntry>) -> Self {
        Self { entries }
    }

    pub fn entries(&self) -> &[JournalEntry] {
        &self.entries
    }
}
