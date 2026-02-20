use chrono::TimeDelta;

use crate::types::TimeUnit;

/// Represents a fixed amount of time in a given unit (e.g. hours, minutes).
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct Duration {
    amount: u64,
    unit: TimeUnit,
}

impl Duration {
    /// Creates a new Duration with the given amount and unit.
    pub const fn of(amount: u64, unit: TimeUnit) -> Self {
        Self { amount, unit }
    }

    /// Creates a new Duration in days.
    pub const fn days(amount: u64) -> Self {
        Self::of(amount, TimeUnit::Day)
    }

    /// Creates a new Duration in hours.
    pub const fn hours(amount: u64) -> Self {
        Self::of(amount, TimeUnit::Hour)
    }

    /// Creates a new Duration in minutes.
    pub const fn minutes(amount: u64) -> Self {
        Self::of(amount, TimeUnit::Minute)
    }

    /// Returns a [`chrono::TimeDelta`] reflecting the duration.
    pub const fn timedelta(&self) -> TimeDelta {
        TimeDelta::seconds(self.seconds() as i64)
    }

    const fn seconds(&self) -> u64 {
        self.amount * self.unit.seconds() as u64
    }
}

impl std::fmt::Display for Duration {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}{}", self.amount, self.unit)
    }
}

impl std::ops::Add for Duration {
    type Output = Duration;

    fn add(self, rhs: Self) -> Self::Output {
        Duration::of(rhs.seconds() + self.seconds(), TimeUnit::Second)
    }
}

#[cfg(test)]
mod test {

    use super::*;

    #[test]
    fn test_timedelta() {
        let d = Duration::of(123, TimeUnit::Second);
        assert_eq!(d.timedelta(), TimeDelta::seconds(123));

        let d = Duration::minutes(123);
        assert_eq!(d.timedelta(), TimeDelta::minutes(123));

        let d = Duration::hours(123);
        assert_eq!(d.timedelta(), TimeDelta::hours(123));

        let d = Duration::of(123, TimeUnit::Day);
        assert_eq!(d.timedelta(), TimeDelta::days(123));

        let d = Duration::of(123, TimeUnit::Month);
        assert_eq!(d.timedelta(), TimeDelta::days(30 * 123));

        let d = Duration::of(123, TimeUnit::Year);
        assert_eq!(d.timedelta(), TimeDelta::days(365 * 123));
    }

    #[test]
    fn test_add() {
        let a = Duration::of(60, TimeUnit::Second);
        let b = Duration::of(30, TimeUnit::Minute);
        let c = Duration::of(2, TimeUnit::Hour);

        assert_eq!(
            Duration::of(151, TimeUnit::Minute).seconds(),
            (a + b + c).seconds()
        );
    }
}
