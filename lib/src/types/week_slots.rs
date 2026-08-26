use chrono::DateTime;
use chrono::Datelike;
use chrono::TimeDelta;
use chrono::TimeZone;
use serde::Deserialize;
use serde::Serialize;

use crate::types::days::DayOfWeek;

/// Represents the day-of-week dimension of an
/// [`Availability`](crate::types::Availability).
///
/// A value describes an inclusive range of days `[start, stop]`.
/// A single day is represented as `WeekSlot { start: d, stop: d }`.
#[derive(Debug, PartialEq, Eq, Clone, Copy, Serialize, Deserialize)]
pub struct WeekSlot {
    /// Inclusive start day
    start: DayOfWeek,
    /// Inclusive end day
    stop: DayOfWeek,
}

impl WeekSlot {
    pub const fn fixed(day: DayOfWeek) -> Self {
        Self {
            start: day,
            stop: day,
        }
    }

    pub const fn range(start: DayOfWeek, stop: DayOfWeek) -> Self {
        Self { start, stop }
    }

    pub const fn workdays() -> Self {
        Self {
            start: DayOfWeek::Mon,
            stop: DayOfWeek::Fri,
        }
    }

    pub const fn weekend() -> Self {
        Self {
            start: DayOfWeek::Sat,
            stop: DayOfWeek::Sun,
        }
    }

    pub const fn full() -> Self {
        Self {
            start: DayOfWeek::Mon,
            stop: DayOfWeek::Sun,
        }
    }

    pub fn matches(&self, day: DayOfWeek) -> bool {
        if self.start <= self.stop {
            (self.start..=self.stop).contains(&day)
        } else {
            day >= self.start || day <= self.stop
        }
    }

    pub fn forward_delta(&self, curr: DayOfWeek) -> i64 {
        if self.matches(curr) {
            0
        } else {
            (self.start as i64 - curr as i64 + 7) % 7
        }
    }

    pub fn matches_chrono<T: TimeZone>(&self, ts: DateTime<T>) -> bool {
        self.matches(ts.weekday().into())
    }

    pub fn forward_delta_chrono<T: TimeZone>(&self, ts: DateTime<T>) -> TimeDelta {
        TimeDelta::days(self.forward_delta(ts.weekday().into()))
    }
}

impl std::fmt::Display for WeekSlot {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.start == self.stop {
            write!(f, "{}", self.start)
        } else {
            write!(f, "{}-{}", self.start, self.stop)
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    mod fixed {

        use super::*;
        use crate::test::d;

        #[test]
        fn test_matches() {
            let sut = WeekSlot::fixed(DayOfWeek::Wed);
            assert!(sut.matches(DayOfWeek::Wed));
            assert!(!sut.matches(DayOfWeek::Tue));
            assert!(!sut.matches(DayOfWeek::Thu));
        }

        #[test]
        fn test_forward_delta() {
            let sut = WeekSlot::fixed(DayOfWeek::Mon);
            assert_eq!(4, sut.forward_delta(DayOfWeek::Thu));

            let sut = WeekSlot::fixed(DayOfWeek::Tue);
            assert_eq!(0, sut.forward_delta(DayOfWeek::Tue));

            let sut = WeekSlot::fixed(DayOfWeek::Wed);
            assert_eq!(5, sut.forward_delta(DayOfWeek::Fri));
        }

        #[test]
        fn test_chrono_interop() {
            let sut = WeekSlot::fixed(DayOfWeek::Tue);
            let input = d(2025, 10, 23, 14, 0, 0);

            assert_eq!(TimeDelta::days(5), sut.forward_delta_chrono(input));

            assert!(sut.matches_chrono(input - TimeDelta::days(2)));

            assert!(!sut.matches_chrono(input));
        }

        #[test]
        fn test_serde_roundtrip() {
            let suts = [
                WeekSlot::fixed(DayOfWeek::Mon),
                WeekSlot::fixed(DayOfWeek::Sun),
            ];
            for sut in suts {
                let json = serde_json::to_string(&sut).unwrap();
                let back: WeekSlot = serde_json::from_str(&json).unwrap();
                assert_eq!(sut, back);
            }
        }
    }

    mod range {
        use super::*;

        #[test]
        fn test_matches_wrap_around() {
            let sut = WeekSlot::range(DayOfWeek::Fri, DayOfWeek::Mon);
            assert!(sut.matches(DayOfWeek::Fri));
            assert!(sut.matches(DayOfWeek::Sat));
            assert!(sut.matches(DayOfWeek::Sun));
            assert!(sut.matches(DayOfWeek::Mon));

            assert!(!sut.matches(DayOfWeek::Thu));
            assert!(!sut.matches(DayOfWeek::Tue));
        }

        #[test]
        fn test_forward_delta() {
            let sut = WeekSlot::range(DayOfWeek::Wed, DayOfWeek::Fri);
            assert_eq!(2, sut.forward_delta(DayOfWeek::Mon));
            assert_eq!(0, sut.forward_delta(DayOfWeek::Thu));
            assert_eq!(4, sut.forward_delta(DayOfWeek::Sat));
        }

        #[test]
        fn test_serde_roundtrip() {
            let suts = [
                WeekSlot::range(DayOfWeek::Mon, DayOfWeek::Fri),
                WeekSlot::range(DayOfWeek::Fri, DayOfWeek::Mon),
            ];
            for sut in suts {
                let json = serde_json::to_string(&sut).unwrap();
                let back: WeekSlot = serde_json::from_str(&json).unwrap();
                assert_eq!(sut, back);
            }
        }
    }
}
