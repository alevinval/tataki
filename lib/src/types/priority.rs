use serde::Deserialize;
use serde::Serialize;

/// Priority enumeration.
/// From most to least priority.
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Serialize, Deserialize)]
pub enum Priority {
    Idle,
    Norm,
    High,
    Crit,
}

impl Priority {
    pub const fn as_str(&self) -> &'static str {
        match self {
            Priority::Idle => "IDLE",
            Priority::Norm => "NORM",
            Priority::High => "HIGH",
            Priority::Crit => "CRIT",
        }
    }
}

impl std::fmt::Display for Priority {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("{}", self.as_str()))
    }
}

#[cfg(test)]
mod test {

    use super::*;

    #[test]
    fn test_priority_ord() {
        assert!(Priority::Crit > Priority::High);
        assert!(Priority::High > Priority::Norm);
        assert!(Priority::Norm > Priority::Idle);
        assert!(Priority::Idle == Priority::Idle);
    }

    #[test]
    fn test_serde_roundtrip() {
        let variants = [
            Priority::Idle,
            Priority::Norm,
            Priority::High,
            Priority::Crit,
        ];
        for sut in variants {
            let json = serde_json::to_string(&sut).unwrap();
            let back: Priority = serde_json::from_str(&json).unwrap();
            assert_eq!(sut, back);
        }
    }
}
