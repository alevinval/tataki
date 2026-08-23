use chrono::DateTime;
use chrono::Local;
use serde::Deserialize;
use serde::Serialize;

use crate::storage::StorageError;
use crate::storage::Store;

#[derive(Debug, PartialEq, Eq, Clone, Copy, Serialize, Deserialize)]
pub enum Action {
    Completed,
    Postponed,
}

#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize)]
pub struct Commit {
    blueprint_id: String,
    committed_at: DateTime<Local>,
    action: Action,
}

impl Commit {
    pub const fn new(blueprint_id: String, committed_at: DateTime<Local>, action: Action) -> Self {
        Self {
            blueprint_id,
            committed_at,
            action,
        }
    }

    pub const fn completed(blueprint_id: String, committed_at: DateTime<Local>) -> Self {
        Self::new(blueprint_id, committed_at, Action::Completed)
    }

    pub const fn postponed(blueprint_id: String, committed_at: DateTime<Local>) -> Self {
        Self::new(blueprint_id, committed_at, Action::Postponed)
    }

    pub fn blueprint_id(&self) -> &str {
        &self.blueprint_id
    }

    pub const fn committed_at(&self) -> DateTime<Local> {
        self.committed_at
    }

    pub const fn action(&self) -> Action {
        self.action
    }
}

/// The record of commits.
///
/// The JSONL file in the store is the durable source of truth across runs;
/// this in-memory list is a live view kept in sync by [`Journal::append`], so
/// callers always see the latest committed entries without reloading.
#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize)]
pub struct Journal {
    commits: Vec<Commit>,
}

impl Journal {
    const FILE: &str = "journal.jsonl";

    pub fn new(commits: Vec<Commit>) -> Self {
        Self { commits }
    }

    pub fn commits(&self) -> &[Commit] {
        &self.commits
    }

    pub fn last_commit_for(&self, blueprint_id: &str) -> Option<&Commit> {
        self.commits
            .iter()
            .rev()
            .find(|commit| commit.blueprint_id == blueprint_id)
    }

    pub fn last_commit(&self) -> Option<&Commit> {
        self.commits.last()
    }

    /// Load the journal from the store. Returns an empty journal if no
    /// commits have been recorded yet.
    pub fn load(store: &Store) -> Result<Self, StorageError> {
        let commits: Vec<Commit> = store.load_all(Self::FILE)?;
        Ok(Self::new(commits))
    }

    /// Append a commit: persist it to the store and record it in memory.
    ///
    /// The store's JSONL file is the durable source of truth; the in-memory
    /// journal is a live view kept in sync here so callers always see the
    /// latest committed entries. If the file append fails, the in-memory
    /// journal is left unchanged.
    pub fn append(&mut self, store: &Store, commit: Commit) -> Result<(), StorageError> {
        store.append(Self::FILE, &commit)?;
        self.commits.push(commit);
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::storage::Store;
    use crate::test::d;
    use crate::test::dir;

    #[test]
    fn test_get_last_commit_for() {
        let ts = d(2025, 10, 23, 14, 0, 0);
        let commit = Commit {
            blueprint_id: "found".into(),
            committed_at: ts,
            action: Action::Completed,
        };
        let sut = Journal::new(vec![commit.clone()]);

        assert_eq!(None, sut.last_commit_for("missing"));
        assert_eq!(Some(&commit), sut.last_commit_for("found"));
        assert_eq!(sut.last_commit_for("found"), sut.last_commit());
    }

    #[test]
    fn test_get_last_commit_for_returns_most_recent() {
        let t1 = d(2025, 10, 23, 14, 0, 0);
        let t2 = d(2025, 10, 24, 14, 0, 0);
        let sut = Journal::new(vec![
            Commit::postponed("a".into(), t1),
            Commit::postponed("b".into(), t1),
            Commit::completed("a".into(), t2),
        ]);

        let last = sut.last_commit_for("a").unwrap();
        assert_eq!(last.action(), Action::Completed);
        assert_eq!(last.committed_at(), t2);
        assert_eq!(sut.last_commit_for("b").unwrap().committed_at(), t1);
        assert_eq!(None, sut.last_commit_for("c"));
    }

    #[test]
    fn test_empty_journal() {
        let sut = Journal::new(vec![]);
        assert_eq!(None, sut.last_commit());
        assert_eq!(None, sut.last_commit_for("a"));
    }

    #[test]
    fn test_commit_constructors() {
        let ts = d(2025, 10, 23, 14, 0, 0);
        let completed = Commit::completed("a".into(), ts);
        let postponed = Commit::postponed("a".into(), ts);
        assert_eq!("a", completed.blueprint_id());
        assert_eq!(ts, completed.committed_at());
        assert_eq!(Action::Completed, completed.action());
        assert_eq!(Action::Postponed, postponed.action());
    }

    #[test]
    fn test_append_updates_memory_and_file() {
        let store = Store::open(dir("journal_append"));
        let ts = d(2025, 10, 23, 14, 0, 0);
        let mut sut = Journal::new(vec![]);
        sut.append(&store, Commit::completed("1".into(), ts))
            .unwrap();
        sut.append(&store, Commit::postponed("2".into(), ts))
            .unwrap();

        // Append keeps the in-memory view in sync.
        assert_eq!(
            Some(&Commit::completed("1".into(), ts)),
            sut.last_commit_for("1")
        );
        assert_eq!(Some(&Commit::postponed("2".into(), ts)), sut.last_commit());

        // And the commit is durable: a fresh load sees it too.
        let reloaded = Journal::load(&store).unwrap();
        assert_eq!(
            Some(&Commit::postponed("2".into(), ts)),
            reloaded.last_commit()
        );
        assert_eq!(
            Some(&Commit::completed("1".into(), ts)),
            reloaded.last_commit_for("1")
        );
    }

    #[test]
    fn test_load_missing_file_is_empty() {
        let store = Store::open(dir("journal_missing"));
        let sut = Journal::load(&store).unwrap();
        assert_eq!(None, sut.last_commit());
    }

    #[test]
    fn test_serde_roundtrip() {
        let ts = d(2025, 10, 23, 14, 0, 0);
        let sut = Journal::new(vec![
            Commit::completed("a".into(), ts),
            Commit::postponed("b".into(), ts),
        ]);
        let json = serde_json::to_string(&sut).unwrap();
        let back: Journal = serde_json::from_str(&json).unwrap();
        assert_eq!(sut, back);
    }
}
