use chrono::DateTime;
use chrono::Local;

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum Action {
    Completed,
    Postponed,
}

#[derive(Debug, PartialEq, Eq, Clone)]
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

pub struct Journal {
    commits: Vec<Commit>,
}

impl Journal {
    pub fn new(commits: Vec<Commit>) -> Self {
        Self { commits }
    }

    pub fn commits(&self) -> &[Commit] {
        &self.commits
    }

    pub fn push(&mut self, commit: Commit) {
        self.commits.push(commit);
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
}

#[cfg(test)]
mod test {

    use super::*;
    use crate::test::d;

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
    fn test_push_appends_to_end() {
        let mut sut = Journal::new(vec![]);
        let commit = Commit::completed("a".into(), d(2025, 10, 23, 14, 0, 0));
        sut.push(commit.clone());
        assert_eq!(vec![commit.clone()], sut.commits().to_vec());
        assert_eq!(Some(&commit), sut.last_commit());
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
}
