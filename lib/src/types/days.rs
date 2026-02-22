use chrono::Weekday;

/// Models the days of the week.
///
/// The natural ordering follows the alphabetical order based on variant names.
#[derive(Debug, PartialEq, Eq, Ord, PartialOrd, Clone, Copy)]
pub enum DayOfWeek {
    Mon,
    Tue,
    Wed,
    Thu,
    Fri,
    Sat,
    Sun,
}

impl DayOfWeek {
    pub const fn as_str(&self) -> &'static str {
        match self {
            DayOfWeek::Mon => "Mon",
            DayOfWeek::Tue => "Tue",
            DayOfWeek::Wed => "Wed",
            DayOfWeek::Thu => "Thu",
            DayOfWeek::Fri => "Fri",
            DayOfWeek::Sat => "Sat",
            DayOfWeek::Sun => "Sun",
        }
    }
}

impl std::fmt::Display for DayOfWeek {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

impl From<Weekday> for DayOfWeek {
    fn from(value: Weekday) -> Self {
        match value {
            Weekday::Mon => Self::Mon,
            Weekday::Tue => Self::Tue,
            Weekday::Wed => Self::Wed,
            Weekday::Thu => Self::Thu,
            Weekday::Fri => Self::Fri,
            Weekday::Sat => Self::Sat,
            Weekday::Sun => Self::Sun,
        }
    }
}

#[cfg(test)]
mod test {

    use super::*;

    #[test]
    fn test_dayofweek_ord() {
        assert!(DayOfWeek::Mon < DayOfWeek::Tue);
        assert!(DayOfWeek::Tue < DayOfWeek::Wed);
        assert!(DayOfWeek::Wed < DayOfWeek::Thu);
        assert!(DayOfWeek::Thu < DayOfWeek::Fri);
        assert!(DayOfWeek::Fri < DayOfWeek::Sat);
        assert!(DayOfWeek::Sat < DayOfWeek::Sun);
        assert!(DayOfWeek::Sun == DayOfWeek::Sun);
    }
}
