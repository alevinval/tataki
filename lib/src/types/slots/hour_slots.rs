use chrono::DateTime;
use chrono::TimeDelta;
use chrono::TimeZone;
use chrono::Timelike;
use serde::Deserialize;
use serde::Serialize;

/// Represents a specific hour or hour range in a day (0-23).
///
/// Used to specify when a [`Blueprint`](crate::types::Blueprint) has
/// affinity and should be materialized on particular hours of the day.
#[derive(Debug, PartialEq, Eq, Clone, Copy, Serialize, Deserialize)]
pub enum HourSlot {
    /// A specific hour of the day.
    Fixed { hour: u32 },

    /// An inclusive range of hours `[start, stop]`
    Range { start: u32, stop: u32 },
}

impl HourSlot {
    /// Returns true if the given hour (0-23) falls within this slot.
    pub fn matches(&self, hour: u32) -> bool {
        debug_assert!(hour < 24, "hour must be <24, instead it was {hour}");

        match self {
            HourSlot::Fixed { hour: h } => *h == hour,
            HourSlot::Range { start, stop } => {
                if start < stop {
                    (*start..=*stop).contains(&hour)
                } else {
                    hour >= *start || hour <= *stop
                }
            }
        }
    }

    /// Computes the forward delta in hours.
    /// When the input does not fit the slot, it computes the delta till the
    /// next day.
    pub const fn fwd_delta(&self, curr: u32) -> i64 {
        let pivot = match self {
            HourSlot::Fixed { hour } => {
                if curr <= *hour {
                    *hour
                } else {
                    *hour + 24
                }
            }
            HourSlot::Range { start, stop } => {
                if curr <= *start {
                    *start
                } else if curr > *stop {
                    *start + 24
                } else {
                    curr
                }
            }
        };
        pivot as i64 - curr as i64
    }

    /// Computes the backward delta in hours.
    /// For ranged slots, it snaps to the start.
    pub fn bwd_delta(&self, curr: u32) -> i64 {
        (match self {
            HourSlot::Fixed { hour } => {
                if curr <= *hour {
                    24 - hour - curr
                } else {
                    curr - hour
                }
            }
            HourSlot::Range { start, .. } => {
                if curr <= *start {
                    24 - start - curr
                } else {
                    curr - *start
                }
            }
        }) as i64
    }

    pub fn matches_chrono<T: TimeZone>(&self, ts: DateTime<T>) -> bool {
        self.matches(ts.hour())
    }

    pub fn fwd_delta_chrono<T: TimeZone>(&self, ts: DateTime<T>) -> TimeDelta {
        TimeDelta::hours(self.fwd_delta(ts.hour()))
    }

    pub fn bwd_delta_chrono<T: TimeZone>(&self, ts: DateTime<T>) -> TimeDelta {
        TimeDelta::hours(self.bwd_delta(ts.hour()))
    }

    /// Backward delta to snap to the beginning of the slot.
    /// Returns hours to go back to the start of the most recent slot.
    pub fn backward_delta_chrono<T: TimeZone>(&self, src: DateTime<T>) -> TimeDelta {
        let curr = src.hour();
        let delta = match self {
            HourSlot::Fixed { hour } => (curr as i64 - *hour as i64 + 24) % 24,
            HourSlot::Range { start, .. } => {
                if curr < *start {
                    (curr as i64 - *start as i64 + 24) % 24
                } else {
                    curr as i64 - *start as i64
                }
            }
        };

        TimeDelta::hours(delta)
    }

    /// Backward delta to snap to the beginning of the slot.
    /// Returns 0 if currently inside the slot, otherwise hours to go back to
    /// the start of the most recent slot.
    pub fn backward_delta(&self, src: u32) -> i64 {
        if self.matches(src) {
            return 0;
        }

        match self {
            HourSlot::Fixed { hour } => (src as i64 - *hour as i64 + 24) % 24,
            HourSlot::Range { start, .. } => (src as i64 - *start as i64 + 24) % 24,
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
        fn test_fwd_delta() {
            let sut = HourSlot::Fixed { hour: 12 };
            assert_eq!(4, sut.fwd_delta(8));

            let sut = HourSlot::Fixed { hour: 12 };
            assert_eq!(0, sut.fwd_delta(12));

            let sut = HourSlot::Fixed { hour: 12 };
            assert_eq!(22, sut.fwd_delta(14));
        }

        #[test]
        fn test_bwd_delta_chrono() {
            let sut = HourSlot::Fixed { hour: 12 };

            let input = d(2025, 10, 23, 14, 0, 0);

            assert_eq!(TimeDelta::hours(2), sut.bwd_delta_chrono(input));
        }

        #[test]
        fn test_backward_delta_chrono() {
            let sut = HourSlot::Fixed { hour: 12 };

            // Inside slot - no backward delta needed
            let input = d(2025, 10, 23, 12, 0, 0);
            assert_eq!(TimeDelta::hours(0), sut.backward_delta_chrono(input));

            // Outside slot - go back to 12:00 (2 hours)
            let input = d(2025, 10, 23, 14, 0, 0);
            assert_eq!(TimeDelta::hours(2), sut.backward_delta_chrono(input));

            // Outside slot - go back to yesterday's 12:00 (22 hours)
            let input = d(2025, 10, 23, 10, 0, 0);
            assert_eq!(TimeDelta::hours(22), sut.backward_delta_chrono(input));
        }

        #[test]
        fn test_chrono_interop() {
            let sut = HourSlot::Fixed { hour: 12 };
            let input = d(2025, 10, 23, 14, 0, 0);

            assert_eq!(TimeDelta::hours(22), sut.fwd_delta_chrono(input));

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
        fn test_fwd_delta() {
            let sut = HourSlot::Range {
                start: 12,
                stop: 15,
            };
            assert_eq!(4, sut.fwd_delta(8));

            let sut = HourSlot::Range {
                start: 12,
                stop: 15,
            };
            assert_eq!(0, sut.fwd_delta(12));

            let sut = HourSlot::Range {
                start: 12,
                stop: 15,
            };
            assert_eq!(0, sut.fwd_delta(14));

            let sut = HourSlot::Range {
                start: 12,
                stop: 15,
            };
            assert_eq!(18, sut.fwd_delta(18));
        }

        #[test]
        fn test_bwd_delta_chrono() {
            let sut = HourSlot::Range {
                start: 12,
                stop: 15,
            };
            assert_eq!(4, sut.bwd_delta(8));

            let sut = HourSlot::Range {
                start: 12,
                stop: 15,
            };
            assert_eq!(0, sut.bwd_delta(12));

            let sut = HourSlot::Range {
                start: 12,
                stop: 15,
            };
            assert_eq!(2, sut.bwd_delta(14));

            let sut = HourSlot::Range {
                start: 12,
                stop: 15,
            };
            assert_eq!(6, sut.bwd_delta(18));
        }

        #[test]
        fn test_backward_delta_chrono() {
            let sut = HourSlot::Range {
                start: 12,
                stop: 15,
            };

            // Inside range - snap back to slot start (12:00)
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
