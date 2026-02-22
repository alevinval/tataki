/// Models the unit of time.
///
/// The natural ordering corresponds to duration magnitude:
/// `Second < Minute < Hour < Day < Month < Year`.
#[derive(Debug, PartialEq, Eq, Ord, PartialOrd, Clone, Copy)]
pub enum TimeUnit {
    /// Represents a 1 second duration.
    Second,
    /// Represents a 60 second duration.
    Minute,
    /// Represents a 60 minute duration.
    Hour,
    /// Represents a 24 hour duration.
    Day,
    /// Represents a 30 day duration.
    Month,
    /// Represents a 365 day duration.
    Year,
}

impl TimeUnit {
    pub const fn as_str(&self) -> &'static str {
        match self {
            TimeUnit::Second => "s",
            TimeUnit::Minute => "min",
            TimeUnit::Hour => "h",
            TimeUnit::Day => "d",
            TimeUnit::Month => "mo",
            TimeUnit::Year => "y",
        }
    }

    pub const fn seconds(&self) -> i64 {
        match self {
            TimeUnit::Second => 1,
            TimeUnit::Minute => 60,
            TimeUnit::Hour => 3600,
            TimeUnit::Day => 86400,
            TimeUnit::Month => 2592000,
            TimeUnit::Year => 31536000,
        }
    }
}

impl std::fmt::Display for TimeUnit {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

#[cfg(test)]
mod test {

    use super::*;

    #[test]
    fn test_timeunit_ord() {
        assert!(TimeUnit::Second < TimeUnit::Minute);
        assert!(TimeUnit::Minute < TimeUnit::Hour);
        assert!(TimeUnit::Hour < TimeUnit::Day);
        assert!(TimeUnit::Day < TimeUnit::Month);
        assert!(TimeUnit::Month < TimeUnit::Year);
        assert!(TimeUnit::Year == TimeUnit::Year);
    }

    #[test]
    fn test_seconds() {
        assert_eq!(1, TimeUnit::Second.seconds());
        assert_eq!(60, TimeUnit::Minute.seconds());
        assert_eq!(3600, TimeUnit::Hour.seconds());
        assert_eq!(86400, TimeUnit::Day.seconds());
        assert_eq!(2592000, TimeUnit::Month.seconds());
        assert_eq!(31536000, TimeUnit::Year.seconds());
    }
}
