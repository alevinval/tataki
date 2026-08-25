use chrono::DateTime;
use chrono::TimeDelta;
use chrono::TimeZone;
use chrono::Timelike;
use serde::Deserialize;
use serde::Serialize;

/// Represents the hour-of-day dimension of an
/// [`Availability`](crate::types::Availability).
///
/// A value may describe either a single hour or an inclusive range of hours.
#[derive(Debug, PartialEq, Eq, Clone, Copy, Serialize, Deserialize)]
pub enum HourSlot {
    /// A specific hour of the day.
    Fixed { hour: u32 },

    /// An inclusive range of hours `[start, stop]`
    Range { start: u32, stop: u32 },
}

impl HourSlot {
    pub fn matches(&self, hour: u32) -> bool {
        debug_assert!(hour < 24, "hour must be <24, instead it was {hour}");

        match self {
            Self::Fixed { hour: h } => *h == hour,
            Self::Range { start, stop } if start <= stop => (*start..=*stop).contains(&hour),
            Self::Range { start, stop } => hour >= *start || hour <= *stop,
        }
    }

    pub fn forward_delta(&self, curr: u32) -> i64 {
        if self.matches(curr) {
            0
        } else {
            (self.start_hour() as i64 - curr as i64 + 24) % 24
        }
    }

    pub fn backward_delta(&self, curr: u32) -> i64 {
        (curr as i64 - self.start_hour() as i64 + 24) % 24
    }

    pub fn matches_chrono<T: TimeZone>(&self, ts: DateTime<T>) -> bool {
        self.matches(ts.hour())
    }

    pub fn forward_delta_chrono<T: TimeZone>(&self, ts: DateTime<T>) -> TimeDelta {
        TimeDelta::hours(self.forward_delta(ts.hour()))
    }

    pub fn backward_delta_chrono<T: TimeZone>(&self, ts: DateTime<T>) -> TimeDelta {
        TimeDelta::hours(self.backward_delta(ts.hour()))
    }

    const fn start_hour(&self) -> u32 {
        match self {
            Self::Fixed { hour } | Self::Range { start: hour, .. } => *hour,
        }
    }
}

impl std::fmt::Display for HourSlot {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            HourSlot::Fixed { hour } => write!(f, "{:02}:00", hour),
            HourSlot::Range { start, stop } => write!(f, "{:02}:00-{:02}:00", start, stop),
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
            let sut = HourSlot::Fixed { hour: 12 };
            assert!(sut.matches(12));
            assert!(!sut.matches(11));
            assert!(!sut.matches(13));
        }

        #[test]
        fn test_forward_delta() {
            let sut = HourSlot::Fixed { hour: 12 };
            assert_eq!(4, sut.forward_delta(8));

            let sut = HourSlot::Fixed { hour: 12 };
            assert_eq!(0, sut.forward_delta(12));

            let sut = HourSlot::Fixed { hour: 12 };
            assert_eq!(22, sut.forward_delta(14));
        }

        #[test]
        fn test_backward_delta_chrono() {
            let sut = HourSlot::Fixed { hour: 12 };

            // Inside range - no backward delta needed
            let input = d(2025, 10, 23, 12, 0, 0);
            assert_eq!(TimeDelta::hours(0), sut.backward_delta_chrono(input));

            // Outside range - go back to 12:00 (2 hours)
            let input = d(2025, 10, 23, 14, 0, 0);
            assert_eq!(TimeDelta::hours(2), sut.backward_delta_chrono(input));

            // Outside range - go back to yesterday's 12:00 (22 hours)
            let input = d(2025, 10, 23, 10, 0, 0);
            assert_eq!(TimeDelta::hours(22), sut.backward_delta_chrono(input));
        }

        #[test]
        fn test_chrono_interop() {
            let sut = HourSlot::Fixed { hour: 12 };
            let input = d(2025, 10, 23, 14, 0, 0);

            assert_eq!(TimeDelta::hours(22), sut.forward_delta_chrono(input));

            assert!(sut.matches_chrono(input - TimeDelta::hours(2)));

            assert!(!sut.matches_chrono(input));
        }

        #[test]
        fn test_serde_roundtrip() {
            let sut = HourSlot::Fixed { hour: 9 };
            let json = serde_json::to_string(&sut).unwrap();
            let back: HourSlot = serde_json::from_str(&json).unwrap();
            assert_eq!(sut, back);
        }
    }

    mod range {
        use super::*;
        use crate::test::d;

        #[test]
        fn test_matches() {
            let sut = HourSlot::Range { start: 8, stop: 3 };
            assert!(sut.matches(8));
            assert!(sut.matches(23));
            assert!(sut.matches(0));
            assert!(sut.matches(3));

            assert!(!sut.matches(4));
            assert!(!sut.matches(7));
        }

        #[test]
        fn test_forward_delta() {
            let sut = HourSlot::Range {
                start: 12,
                stop: 15,
            };
            assert_eq!(4, sut.forward_delta(8));

            let sut = HourSlot::Range {
                start: 12,
                stop: 15,
            };
            assert_eq!(0, sut.forward_delta(12));

            let sut = HourSlot::Range {
                start: 12,
                stop: 15,
            };
            assert_eq!(0, sut.forward_delta(14));

            let sut = HourSlot::Range {
                start: 12,
                stop: 15,
            };
            assert_eq!(18, sut.forward_delta(18));
        }

        #[test]
        fn test_backward_delta() {
            let sut = HourSlot::Range {
                start: 12,
                stop: 15,
            };
            assert_eq!(20, sut.backward_delta(8));
            assert_eq!(0, sut.backward_delta(12));
            assert_eq!(2, sut.backward_delta(14));
            assert_eq!(6, sut.backward_delta(18));
        }

        #[test]
        fn test_backward_delta_chrono() {
            let sut = HourSlot::Range {
                start: 12,
                stop: 15,
            };

            // Inside range - snap back to the range start (12:00)
            let input = d(2025, 10, 23, 14, 0, 0);
            assert_eq!(TimeDelta::hours(2), sut.backward_delta_chrono(input));

            // Outside range (before) - go back to yesterday's 12:00 = 22 hours
            let input = d(2025, 10, 23, 10, 0, 0);
            assert_eq!(TimeDelta::hours(22), sut.backward_delta_chrono(input));

            // Outside range (after) - go back to 12:00 same day = 6 hours
            let input = d(2025, 10, 23, 18, 0, 0);
            assert_eq!(TimeDelta::hours(6), sut.backward_delta_chrono(input));
        }

        #[test]
        fn test_serde_roundtrip() {
            let suts = [
                HourSlot::Range { start: 8, stop: 12 },
                HourSlot::Range { start: 20, stop: 2 },
            ];
            for sut in suts {
                let json = serde_json::to_string(&sut).unwrap();
                let back: HourSlot = serde_json::from_str(&json).unwrap();
                assert_eq!(sut, back);
            }
        }
    }
}
