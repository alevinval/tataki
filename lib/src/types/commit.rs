use chrono::DateTime;
use chrono::Local;
use serde::Deserialize;
use serde::Serialize;

use crate::types::Action;

#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize)]
pub struct Commit {
    action: Action,
    blueprint_id: String,
    committed_at: DateTime<Local>,
}

impl Commit {
    pub const fn completed(blueprint_id: String, committed_at: DateTime<Local>) -> Self {
        Self::new(blueprint_id, committed_at, Action::Completed)
    }

    pub const fn postponed(blueprint_id: String, committed_at: DateTime<Local>) -> Self {
        Self::new(blueprint_id, committed_at, Action::Postponed)
    }

    const fn new(blueprint_id: String, committed_at: DateTime<Local>, action: Action) -> Self {
        Self {
            blueprint_id,
            committed_at,
            action,
        }
    }

    pub const fn action(&self) -> Action {
        self.action
    }

    pub fn blueprint_id(&self) -> &str {
        &self.blueprint_id
    }

    pub const fn committed_at(&self) -> DateTime<Local> {
        self.committed_at
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::test::d;

    #[test]
    fn test_commit_completed() {
        let ts = d(2025, 10, 23, 14, 0, 0);
        let commit = Commit::completed("a".into(), ts);
        assert_eq!("a", commit.blueprint_id());
        assert_eq!(ts, commit.committed_at());
        assert_eq!(Action::Completed, commit.action());
    }

    #[test]
    fn test_commit_postponed() {
        let ts = d(2025, 10, 23, 14, 0, 0);
        let commit = Commit::postponed("a".into(), ts);
        assert_eq!("a", commit.blueprint_id());
        assert_eq!(ts, commit.committed_at());
        assert_eq!(Action::Postponed, commit.action());
    }
}
