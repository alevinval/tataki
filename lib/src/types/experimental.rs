use chrono::DateTime;
use chrono::Local;

pub mod book;
pub mod plan;
pub mod plan_entry;

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
