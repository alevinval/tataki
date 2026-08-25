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
    use crate::test::dsl_book;
    use crate::test::tmpdir;
    use crate::types::experimental::book::Book;

    #[test]
    fn test_load_save_roundtrip() {
        let dir = tmpdir("book_load_save");
        let store = Store::open(&dir);
        let sut = dsl_book(&["1 CRIT ^1 1h 08:00"]);
        sut.save(&store).unwrap();
        assert_eq!(sut, Book::load(&store).unwrap());
    }

    #[test]
    fn test_serde_roundtrip() {
        let sut = dsl_book(&["1 CRIT ^1 1h 08:00", "2 NORM ^1d 1h Mon-Fri"]);
        let json = serde_json::to_string(&sut).unwrap();
        let back: Book = serde_json::from_str(&json).unwrap();
        assert_eq!(sut, back);
    }
}
