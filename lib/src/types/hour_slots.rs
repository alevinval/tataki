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
