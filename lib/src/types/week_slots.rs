use serde::Deserialize;
use serde::Serialize;

use crate::types::days::DayOfWeek;

/// Represents the day-of-week dimension of an
/// [`Availability`](crate::types::Availability).
///
/// A value describes an inclusive range of days `[from, to]`.
/// A single day is represented as `WeekSlot { from: d, to: d }`.
#[derive(Debug, PartialEq, Eq, Clone, Copy, Serialize, Deserialize)]
pub struct WeekSlot {
    /// Inclusive start day
    from: DayOfWeek,
    /// Inclusive end day
    to: DayOfWeek,
}

impl WeekSlot {
    pub const fn fixed(day: DayOfWeek) -> Self {
        Self { from: day, to: day }
    }

    pub const fn range(from: DayOfWeek, to: DayOfWeek) -> Self {
        Self { from, to }
    }

    pub const fn workdays() -> Self {
        Self {
            from: DayOfWeek::Mon,
            to: DayOfWeek::Fri,
        }
    }

    pub const fn weekend() -> Self {
        Self {
            from: DayOfWeek::Sat,
            to: DayOfWeek::Sun,
        }
    }

    pub const fn full() -> Self {
        Self {
            from: DayOfWeek::Mon,
            to: DayOfWeek::Sun,
        }
    }

    pub fn matches(&self, day: DayOfWeek) -> bool {
        if self.from <= self.to {
            (self.from..=self.to).contains(&day)
        } else {
            day >= self.from || day <= self.to
        }
    }
}

impl std::fmt::Display for WeekSlot {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.from == self.to {
            write!(f, "{}", self.from)
        } else {
            write!(f, "{}-{}", self.from, self.to)
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
