use chrono::DateTime;
use chrono::TimeDelta;
use chrono::TimeZone;
use chrono::Timelike;
use serde::Deserialize;
use serde::Serialize;

/// Represents the hour-of-day dimension of an
/// [`Availability`](crate::types::Availability).
///
/// A value describes an inclusive range of hours `[start, end]`.
/// A single hour is represented as `HourSlot { start: h, end: h }`.
#[derive(Debug, PartialEq, Eq, Clone, Copy, Serialize, Deserialize)]
pub struct HourSlot {
    /// Inclusive start hour
    start: u32,
    /// Inclusive end hour
    stop: u32,
}

impl HourSlot {
    /// Constructs a slot for a single hour.
    pub const fn fixed(hour: u32) -> Self {
        Self {
            start: hour,
            stop: hour,
        }
    }

    /// Constructs a slot for an inclusive range of hours `[start, stop]`.
    pub const fn range(start: u32, stop: u32) -> Self {
        Self { start, stop }
    }

    /// Inclusive start hour.
    pub const fn start(&self) -> u32 {
        self.start
    }

    /// Inclusive end hour.
    pub const fn stop(&self) -> u32 {
        self.stop
    }

    pub fn matches(&self, hour: u32) -> bool {
        debug_assert!(hour < 24, "hour must be <24, instead it was {hour}");

        if self.start <= self.stop {
            (self.start..=self.stop).contains(&hour)
        } else {
            hour >= self.start || hour <= self.stop
        }
    }

    pub fn forward_delta(&self, curr: u32) -> i64 {
        if self.matches(curr) {
            0
        } else {
            (self.start as i64 - curr as i64 + 24) % 24
        }
    }

    pub fn backward_delta(&self, curr: u32) -> i64 {
        (curr as i64 - self.start as i64 + 24) % 24
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
}

impl std::fmt::Display for HourSlot {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.start == self.stop {
            write!(f, "{:02}:00", self.start)
        } else {
            write!(f, "{:02}:00-{:02}:00", self.start, self.stop)
        }
    }
}

#[cfg(test)]
mod test {

    use super::*;
    use crate::test::d;

    mod fixed {

        use super::*;

        #[test]
        fn test_matches() {
            let sut = HourSlot::fixed(12);
            assert!(sut.matches(12));
            assert!(!sut.matches(11));
            assert!(!sut.matches(13));
        }

        #[test]
        fn test_forward_delta() {
            let sut = HourSlot::fixed(12);
            assert_eq!(4, sut.forward_delta(8));

            let sut = HourSlot::fixed(12);
            assert_eq!(0, sut.forward_delta(12));

            let sut = HourSlot::fixed(12);
            assert_eq!(22, sut.forward_delta(14));
        }

        #[test]
        fn test_backward_delta_chrono() {
            let sut = HourSlot::fixed(12);

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
            let sut = HourSlot::fixed(12);
            let input = d(2025, 10, 23, 14, 0, 0);

            assert_eq!(TimeDelta::hours(22), sut.forward_delta_chrono(input));

            assert!(sut.matches_chrono(input - TimeDelta::hours(2)));

            assert!(!sut.matches_chrono(input));
        }

        #[test]
        fn test_serde_roundtrip() {
            let sut = HourSlot::fixed(9);
            let json = serde_json::to_string(&sut).unwrap();
            let back: HourSlot = serde_json::from_str(&json).unwrap();
            assert_eq!(sut, back);
        }
    }

    mod range {
        use super::*;

        #[test]
        fn test_matches() {
            let sut = HourSlot::range(8, 3);
            assert!(sut.matches(8));
            assert!(sut.matches(23));
            assert!(sut.matches(0));
            assert!(sut.matches(3));

            assert!(!sut.matches(4));
            assert!(!sut.matches(7));
        }

        #[test]
        fn test_forward_delta() {
            let sut = HourSlot::range(12, 15);
            assert_eq!(4, sut.forward_delta(8));

            let sut = HourSlot::range(12, 15);
            assert_eq!(0, sut.forward_delta(12));

            let sut = HourSlot::range(12, 15);
            assert_eq!(0, sut.forward_delta(14));

            let sut = HourSlot::range(12, 15);
            assert_eq!(18, sut.forward_delta(18));
        }

        #[test]
        fn test_backward_delta() {
            let sut = HourSlot::range(12, 15);
            assert_eq!(20, sut.backward_delta(8));
            assert_eq!(0, sut.backward_delta(12));
            assert_eq!(2, sut.backward_delta(14));
            assert_eq!(6, sut.backward_delta(18));
        }

        #[test]
        fn test_backward_delta_chrono() {
            let sut = HourSlot::range(12, 15);

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
            let suts = [HourSlot::range(8, 12), HourSlot::range(20, 2)];
            for sut in suts {
                let json = serde_json::to_string(&sut).unwrap();
                let back: HourSlot = serde_json::from_str(&json).unwrap();
                assert_eq!(sut, back);
            }
        }
    }
}
