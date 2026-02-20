/// Priority enumeration.
/// From most to least priority.
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
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
}
