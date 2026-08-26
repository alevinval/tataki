use serde::Deserialize;
use serde::Serialize;

/// Priority.
///
/// A lower `value` means higher priority (`P(0)` > `P(1)` > ...).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Priority {
    rank: u8,
}

impl Priority {
    /// Construct a priority from its numeric value.
    pub const fn of(rank: u8) -> Self {
        Self { rank }
    }
}

impl PartialOrd for Priority {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Priority {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        other.rank.cmp(&self.rank)
    }
}

impl std::fmt::Display for Priority {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "P{}", self.rank)
    }
}

#[cfg(test)]
mod test {

    use super::*;

    #[test]
    fn test_priority_ord() {
        assert!(Priority::of(0) > Priority::of(1));
        assert!(Priority::of(1) > Priority::of(2));
        assert!(Priority::of(2) > Priority::of(3));
        assert!(Priority::of(3) == Priority::of(3));
    }

    #[test]
    fn test_serde_roundtrip() {
        let variants = [
            Priority::of(0),
            Priority::of(1),
            Priority::of(2),
            Priority::of(3),
        ];
        for sut in variants {
            let json = serde_json::to_string(&sut).unwrap();
            let back: Priority = serde_json::from_str(&json).unwrap();
            assert_eq!(sut, back);
        }
    }

    #[test]
    fn test_display() {
        assert_eq!(Priority::of(0).to_string(), "P0");
        assert_eq!(Priority::of(3).to_string(), "P3");
        assert_eq!(Priority::of(5).to_string(), "P5");
    }
}
