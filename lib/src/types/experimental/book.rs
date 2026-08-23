use std::cmp;

use chrono::DateTime;
use chrono::Local;
use chrono::TimeDelta;
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

    pub fn min_fwd_delta_chrono(&self, ts: DateTime<Local>) -> Option<TimeDelta> {
        self.blueprints
            .iter()
            .map(|blueprint| blueprint.preferred_slot().fwd_delta_chrono(ts))
            .filter(|delta| !delta.is_zero())
            .min()
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
    use chrono::TimeDelta;

    use crate::storage::Store;
    use crate::test::d;
    use crate::test::tmpdir;
    use crate::types::Blueprint;
    use crate::types::Duration;
    use crate::types::HourSlot;
    use crate::types::Priority;
    use crate::types::Recurrence;
    use crate::types::Slot;
    use crate::types::TimeUnit;
    use crate::types::experimental::book::Book;

    #[test]
    fn test_min_fwd_delta_chrono() {
        let eight_am = Slot::Hour(HourSlot::Fixed { hour: 8 });
        let morning = Slot::Hour(HourSlot::Range { start: 8, stop: 12 });
        let daily = Recurrence::Period {
            spacing: Duration::of(1, TimeUnit::Day),
        };

        let one_hour = Duration::of(1, TimeUnit::Hour);

        let sut = Book::new(vec![
            Blueprint::new(
                "1".to_string(),
                "Task A".to_string(),
                one_hour,
                Priority::Crit,
                daily,
                eight_am,
            ),
            Blueprint::new(
                "2".to_string(),
                "Task B".to_string(),
                one_hour,
                Priority::Norm,
                daily,
                morning,
            ),
        ]);

        let ts = d(2025, 10, 23, 14, 0, 0);
        assert_eq!(Some(TimeDelta::hours(18)), sut.min_fwd_delta_chrono(ts));
    }

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
            Slot::Hour(HourSlot::Fixed { hour: 8 }),
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
                Slot::Hour(HourSlot::Fixed { hour: 8 }),
            ),
            Blueprint::new(
                "2".to_string(),
                "Task B".to_string(),
                one_hour,
                Priority::Norm,
                Recurrence::Period {
                    spacing: Duration::of(1, TimeUnit::Day),
                },
                Slot::Week(crate::types::WeekSlot::workdays()),
            ),
        ]);
        let json = serde_json::to_string(&sut).unwrap();
        let back: Book = serde_json::from_str(&json).unwrap();
        assert_eq!(sut, back);
    }
}
