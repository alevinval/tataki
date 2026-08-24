use std::cmp;

use serde::Deserialize;
use serde::Serialize;

use crate::sequencer::Sequencer;
use crate::storage::StorageError;
use crate::storage::Store;
use crate::types::Blueprint;
use crate::types::experimental::journal::Journal;

/// Models a collection of blueprints.
#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize)]
pub struct Book {
    blueprints: Vec<Blueprint>,
}

impl Book {
    const FILE: &str = "book.json";

    pub fn new(mut blueprints: Vec<Blueprint>) -> Self {
        blueprints.sort_by_key(|b| cmp::Reverse(b.priority()));
        Self { blueprints }
    }

    pub fn blueprints(&self) -> &[Blueprint] {
        &self.blueprints
    }

    pub fn spawn_sequencers(&self, journal: &Journal) -> Vec<(Blueprint, Sequencer)> {
        self.blueprints
            .iter()
            .map(|bp| (bp.clone(), Sequencer::from(bp, journal)))
            .collect()
    }

    /// Load the book from the store.
    pub fn load(store: &Store) -> Result<Self, StorageError> {
        store.load(Self::FILE)
    }

    /// Save the book to the store, atomically.
    pub fn save(&self, store: &Store) -> Result<(), StorageError> {
        store.save(Self::FILE, self)
    }
}

impl std::fmt::Display for Book {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for bp in &self.blueprints {
            bp.fmt(f)?;
            f.write_str("\n")?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use crate::storage::Store;
    use crate::test::tmpdir;
    use crate::types::Availability;
    use crate::types::Blueprint;
    use crate::types::Duration;
    use crate::types::HourSlot;
    use crate::types::Priority;
    use crate::types::Recurrence;
    use crate::types::TimeUnit;
    use crate::types::WeekSlot;
    use crate::types::experimental::book::Book;

    #[test]
    fn test_load_save_roundtrip() {
        let dir = tmpdir("book_load_save");
        let store = Store::open(&dir);
        let sut = Book::new(vec![Blueprint::new(
            "1".to_string(),
            "Task A".to_string(),
            Duration::of(1, TimeUnit::Hour),
            Priority::Crit,
            Recurrence::Once,
            Availability::new(WeekSlot::full(), HourSlot::Fixed { hour: 8 }),
        )]);
        sut.save(&store).unwrap();
        assert_eq!(sut, Book::load(&store).unwrap());
    }

    #[test]
    fn test_serde_roundtrip() {
        let one_hour = Duration::of(1, TimeUnit::Hour);
        let sut = Book::new(vec![
            Blueprint::new(
                "1".to_string(),
                "Task A".to_string(),
                one_hour,
                Priority::Crit,
                Recurrence::Once,
                Availability::new(WeekSlot::full(), HourSlot::Fixed { hour: 8 }),
            ),
            Blueprint::new(
                "2".to_string(),
                "Task B".to_string(),
                one_hour,
                Priority::Norm,
                Recurrence::Period {
                    spacing: Duration::of(1, TimeUnit::Day),
                },
                Availability::anytime(WeekSlot::workdays()),
            ),
        ]);
        let json = serde_json::to_string(&sut).unwrap();
        let back: Book = serde_json::from_str(&json).unwrap();
        assert_eq!(sut, back);
    }
}
