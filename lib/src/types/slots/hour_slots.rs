use chrono::DateTime;
use chrono::TimeDelta;
use chrono::TimeZone;
use chrono::Timelike;

/// Represents a specific hour or hour range in a day (0-23).
///
/// Used to specify when a [`Blueprint`](crate::types::Blueprint) has
/// affinity and should be materialized on particular hours of the day.
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
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

    /// Computes the forward delta in hours, if the input does not fit the slot,
    /// rather than returning negative, it computes the delta till the next
    /// day.
    pub const fn fwd_delta(&self, src: u32) -> i64 {
        let pivot = match self {
            HourSlot::Fixed { hour } => {
                if src <= *hour {
                    *hour
                } else {
                    *hour + 24
                }
            }
            HourSlot::Range { start, stop } => {
                if src <= *start {
                    *start
                } else if src > *stop {
                    *start + 24
                } else {
                    src
                }
            }
        };
        pivot as i64 - src as i64
    }

    pub fn matches_chrono<T: TimeZone>(&self, src: DateTime<T>) -> bool {
        self.matches(src.hour())
    }

    pub fn fwd_delta_chrono<T: TimeZone>(&self, src: DateTime<T>) -> TimeDelta {
        TimeDelta::hours(self.fwd_delta(src.hour()))
    }

    pub fn backward_delta_chrono<T: TimeZone>(&self, src: DateTime<T>) -> TimeDelta {
        let curr = src.hour();
        let delta = match self {
            HourSlot::Fixed { hour } => curr - hour,
            HourSlot::Range { start, .. } => curr - start,
        };

        TimeDelta::hours(delta as i64)
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

    mod slot_hour {
        use super::*;

        mod fixed {

            use chrono::Utc;

            use super::*;

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
            fn test_chrono_interop() {
                let sut = HourSlot::Fixed { hour: 12 };
                let input = Utc.with_ymd_and_hms(2025, 10, 23, 14, 0, 0).unwrap();

                assert_eq!(TimeDelta::hours(22), sut.fwd_delta_chrono(input));

                assert!(sut.matches_chrono(input - TimeDelta::hours(2)));

                assert!(!sut.matches_chrono(input));
            }

            #[test]
            fn test_backward_delta_chrono() {
                let sut = HourSlot::Fixed { hour: 12 };

                let input = Utc.with_ymd_and_hms(2025, 10, 23, 14, 0, 0).unwrap();

                assert_eq!(TimeDelta::hours(2), sut.backward_delta_chrono(input));
            }
        }

        mod range {
            use chrono::Utc;

            use super::*;

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
            fn test_backward_delta_chrono() {
                let sut = HourSlot::Range {
                    start: 12,
                    stop: 15,
                };

                let input = Utc.with_ymd_and_hms(2025, 10, 23, 14, 0, 0).unwrap();

                assert_eq!(TimeDelta::hours(2), sut.backward_delta_chrono(input));
            }
        }
    }
}
