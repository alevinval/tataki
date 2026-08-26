use serde::Deserialize;
use serde::Serialize;

/// Represents the hour-of-day dimension of an
/// [`Availability`](crate::types::Availability).
///
/// A value describes an inclusive range of hours `[from, to]`.
/// A single hour is represented as `HourSlot { from: h, to: h }`.
#[derive(Debug, PartialEq, Eq, Clone, Copy, Serialize, Deserialize)]
pub struct HourSlot {
    /// Inclusive start hour
    from: u32,
    /// Inclusive end hour
    to: u32,
}

impl HourSlot {
    /// Constructs a slot for a single hour.
    pub const fn fixed(hour: u32) -> Self {
        Self {
            from: hour,
            to: hour,
        }
    }

    /// Constructs a slot for an inclusive range of hours `[from, to]`.
    pub const fn range(from: u32, to: u32) -> Self {
        Self { from, to }
    }

    /// Inclusive start hour.
    pub const fn from(&self) -> u32 {
        self.from
    }

    /// Inclusive end hour.
    pub const fn to(&self) -> u32 {
        self.to
    }

    pub fn matches(&self, hour: u32) -> bool {
        debug_assert!(hour < 24, "hour must be <24, instead it was {hour}");

        if self.from <= self.to {
            (self.from..=self.to).contains(&hour)
        } else {
            hour >= self.from || hour <= self.to
        }
    }
}

impl std::fmt::Display for HourSlot {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.from == self.to {
            write!(f, "{:02}:00", self.from)
        } else {
            write!(f, "{:02}:00-{:02}:00", self.from, self.to)
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
    }

    mod serde {
        use super::*;

        #[test]
        fn test_fixed() {
            let sut = HourSlot::fixed(9);
            let json = serde_json::to_string(&sut).unwrap();
            let back: HourSlot = serde_json::from_str(&json).unwrap();
            assert_eq!(sut, back);
        }

        #[test]
        fn test_range() {
            let suts = [HourSlot::range(8, 12), HourSlot::range(20, 2)];
            for sut in suts {
                let json = serde_json::to_string(&sut).unwrap();
                let back: HourSlot = serde_json::from_str(&json).unwrap();
                assert_eq!(sut, back);
            }
        }
    }
}
