use std::cmp;

use serde::Deserialize;
use serde::Serialize;

use crate::storage::StorageError;
use crate::storage::Store;
use crate::types::Blueprint;

/// Keeps a collection of blueprints.
#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize)]
pub struct Book {
    blueprints: Vec<Blueprint>,
}

impl Book {
    const FILE: &str = "book.json";

    pub fn from_dsl(lines: &[&str]) -> Self {
        Self::new(lines.iter().map(|s| Blueprint::from_dsl(s)).collect())
    }

    pub fn new(blueprints: Vec<Blueprint>) -> Self {
        let mut book = Self { blueprints };
        book.sort();
        book
    }

    pub fn blueprints(&self) -> &[Blueprint] {
        &self.blueprints
    }

    pub fn load(store: &Store) -> Result<Self, StorageError> {
        let mut book: Self = store.load(Self::FILE)?;
        book.sort();
        Ok(book)
    }

    pub fn save(&self, store: &Store) -> Result<(), StorageError> {
        store.save(Self::FILE, self)
    }

    fn sort(&mut self) {
        self.blueprints.sort_by_key(|b| cmp::Reverse(b.priority()));
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
    use crate::types::book::Book;

    #[test]
    fn test_load_save_roundtrip() {
        let dir = tmpdir("book_load_save");
        let store = Store::open(&dir);
        let sut = Book::from_dsl(&["1 CRIT ^1 1h 08:00"]);
        sut.save(&store).unwrap();
        assert_eq!(sut, Book::load(&store).unwrap());
    }

    #[test]
    fn test_load_sorts_blueprints() {
        let dir = tmpdir("book_load_sorts");
        let store = Store::open(&dir);
        let expected = Book::from_dsl(&["1 CRIT ^1 1h 08:00", "2 NORM ^1d 1h Mon-Fri"]);
        let mut unsorted = expected.clone();
        unsorted.blueprints.reverse();
        store.save(Book::FILE, &unsorted).unwrap();
        assert_eq!(Book::load(&store).unwrap(), expected);
    }

    #[test]
    fn test_serde_roundtrip() {
        let sut = Book::from_dsl(&["1 CRIT ^1 1h 08:00", "2 NORM ^1d 1h Mon-Fri"]);
        let json = serde_json::to_string(&sut).unwrap();
        let back: Book = serde_json::from_str(&json).unwrap();
        assert_eq!(sut, back);
    }
}
