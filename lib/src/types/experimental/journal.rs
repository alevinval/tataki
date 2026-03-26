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

    pub fn get_last_commit_for(&self, blueprint_id: &str) -> Option<&Commit> {
        self.commits
            .iter()
            .rev()
            .find(|commit| commit.blueprint_id == blueprint_id)
    }

    pub fn get_last_commit(&self) -> Option<&Commit> {
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

        assert_eq!(None, sut.get_last_commit_for("missing"));
        assert_eq!(Some(&commit), sut.get_last_commit_for("found"));
        assert_eq!(sut.get_last_commit_for("found"), sut.get_last_commit());
    }
}
