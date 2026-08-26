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

        #[test]
        fn test_matches() {
            let sut = WeekSlot::fixed(DayOfWeek::Wed);
            assert!(sut.matches(DayOfWeek::Wed));
            assert!(!sut.matches(DayOfWeek::Tue));
            assert!(!sut.matches(DayOfWeek::Thu));
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
