use chrono::DateTime;
use chrono::Datelike;
use chrono::TimeDelta;
use chrono::TimeZone;

use crate::types::days::DayOfWeek;

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum WeekSlot {
    /// A specific day of the week.
    Fixed { day: DayOfWeek },
    /// An inclusive range of days `[start, stop]`
    Range { start: DayOfWeek, stop: DayOfWeek },
}

impl WeekSlot {
    /// Returns a range that covers all working days.
    pub const fn workdays() -> Self {
        Self::Range {
            start: DayOfWeek::Mon,
            stop: DayOfWeek::Fri,
        }
    }

    /// Returns a range that covers the weekend.
    pub const fn weekend() -> Self {
        Self::Range {
            start: DayOfWeek::Sat,
            stop: DayOfWeek::Sun,
        }
    }

    /// Returns a range that spans a full week
    pub const fn full() -> Self {
        Self::Range {
            start: DayOfWeek::Mon,
            stop: DayOfWeek::Sun,
        }
    }

    /// Returns true if the given day falls within this slot.
    pub fn matches(&self, day: DayOfWeek) -> bool {
        match self {
            WeekSlot::Fixed { day: d } => *d == day,
            WeekSlot::Range { start, stop } => {
                if *start <= *stop {
                    (*start..=*stop).contains(&day)
                } else {
                    day >= *start || day <= *stop
                }
            }
        }
    }

    /// Computes the forward delta in days, if the input does not fit the slot,
    /// rather than returning negative, it computes the delta until the next
    /// week.
    pub const fn fwd_delta(&self, curr: DayOfWeek) -> i64 {
        let curr = curr as u32;
        let pivot = match self {
            Self::Fixed { day } => {
                let day = *day as u32;
                if curr <= day { day } else { day + 7 }
            }
            Self::Range { start, stop } => {
                let (start, stop) = (*start as u32, *stop as u32);
                if curr <= start {
                    start
                } else if curr > stop {
                    start + 7
                } else {
                    curr
                }
            }
        };
        pivot as i64 - curr as i64
    }

    /// Computes the backward delta in days.
    /// For ranged slots, it snaps to the start.
    pub fn bwd_delta(&self, curr: DayOfWeek) -> i64 {
        let curr = curr as u32;
        let pivot = match self {
            WeekSlot::Fixed { day } => *day,
            WeekSlot::Range { start, .. } => *start,
        } as u32;

        (if curr >= pivot {
            curr - pivot
        } else {
            7 + curr - pivot
        }) as i64
    }

    pub fn matches_chrono<T: TimeZone>(&self, ts: DateTime<T>) -> bool {
        self.matches(ts.weekday().into())
    }

    pub fn fwd_delta_chrono<T: TimeZone>(&self, ts: DateTime<T>) -> TimeDelta {
        TimeDelta::days(self.fwd_delta(ts.weekday().into()))
    }

    pub fn bwd_delta_chrono<T: TimeZone>(&self, ts: DateTime<T>) -> TimeDelta {
        TimeDelta::days(self.bwd_delta(ts.weekday().into()))
    }

    /// Backward delta to snap to the beginning of the slot.
    /// Returns days to go back to the start of the most recent slot.
    pub fn backward_delta_chrono<T: TimeZone>(&self, ts: DateTime<T>) -> TimeDelta {
        let curr: DayOfWeek = ts.weekday().into();
        let delta = match self {
            Self::Fixed { day } => (*day as i64 - curr as i64 + 7) % 7,
            Self::Range { start, stop } => {
                let start = *start as i64;
                let stop = *stop as i64;
                let curr = curr as i64;

                if start <= stop {
                    if curr < start {
                        start - curr
                    } else {
                        curr - start
                    }
                } else {
                    (curr - start + 7) % 7
                }
            }
        };
        TimeDelta::days(delta)
    }

    /// Backward delta to snap to the beginning of the slot.
    /// Returns days to go back to the start of the most recent slot.
    pub fn backward_delta(&self, src: DayOfWeek) -> i64 {
        let delta = match self {
            Self::Fixed { day } => (*day as i64 - src as i64 + 7) % 7,
            Self::Range { start, stop } => {
                let start = *start as i64;
                let stop = *stop as i64;
                let src = src as i64;

                if start <= stop {
                    if src < start {
                        start - src
                    } else {
                        src - start
                    }
                } else {
                    (src - start + 7) % 7
                }
            }
        };
        delta
    }
}

impl std::fmt::Display for WeekSlot {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            WeekSlot::Fixed { day } => write!(f, "{}", day),
            WeekSlot::Range { start, stop } => write!(f, "{}-{}", start, stop),
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
            let sut = WeekSlot::Fixed {
                day: DayOfWeek::Wed,
            };
            assert!(sut.matches(DayOfWeek::Wed));
            assert!(!sut.matches(DayOfWeek::Tue));
            assert!(!sut.matches(DayOfWeek::Thu));
        }

        #[test]
        fn test_fwd_delta() {
            let sut = WeekSlot::Fixed {
                day: DayOfWeek::Mon,
            };
            assert_eq!(4, sut.fwd_delta(DayOfWeek::Thu));

            let sut = WeekSlot::Fixed {
                day: DayOfWeek::Tue,
            };
            assert_eq!(0, sut.fwd_delta(DayOfWeek::Tue));

            let sut = WeekSlot::Fixed {
                day: DayOfWeek::Wed,
            };
            assert_eq!(5, sut.fwd_delta(DayOfWeek::Fri));
        }

        #[test]
        fn test_bwd_delta() {
            let sut = WeekSlot::Fixed {
                day: DayOfWeek::Mon,
            };
            assert_eq!(3, sut.bwd_delta(DayOfWeek::Thu));

            let sut = WeekSlot::Fixed {
                day: DayOfWeek::Tue,
            };
            assert_eq!(0, sut.bwd_delta(DayOfWeek::Tue));

            let sut = WeekSlot::Fixed {
                day: DayOfWeek::Wed,
            };
            assert_eq!(2, sut.bwd_delta(DayOfWeek::Fri));
        }

        #[test]
        fn test_chrono_interop() {
            let sut = WeekSlot::Fixed {
                day: DayOfWeek::Tue,
            };
            let input = d(2025, 10, 23, 14, 0, 0);

            assert_eq!(TimeDelta::days(5), sut.fwd_delta_chrono(input));

            assert!(sut.matches_chrono(input - TimeDelta::days(2)));

            assert!(!sut.matches_chrono(input));
        }

        #[test]
        fn test_backward_delta_chrono() {
            let sut = WeekSlot::Fixed {
                day: DayOfWeek::Fri,
            };

            // Inside slot (Friday) - no backward delta needed
            let input = d(2025, 10, 24, 14, 0, 0); // Friday
            assert_eq!(TimeDelta::days(0), sut.backward_delta_chrono(input));

            // Outside slot - go back to last Friday = 1 day
            let input = d(2025, 10, 23, 14, 0, 0); // Thursday
            assert_eq!(TimeDelta::days(1), sut.backward_delta_chrono(input));

            // Outside slot - go back to last Friday = 3 days
            let input = d(2025, 10, 21, 14, 0, 0); // Tuesday
            assert_eq!(TimeDelta::days(3), sut.backward_delta_chrono(input));
        }
    }

    mod range {
        use super::*;
        use crate::test::d;

        #[test]
        fn test_matches_wrap_around() {
            let sut = WeekSlot::Range {
                start: DayOfWeek::Fri,
                stop: DayOfWeek::Mon,
            };
            assert!(sut.matches(DayOfWeek::Fri));
            assert!(sut.matches(DayOfWeek::Sat));
            assert!(sut.matches(DayOfWeek::Sun));
            assert!(sut.matches(DayOfWeek::Mon));

            assert!(!sut.matches(DayOfWeek::Thu));
            assert!(!sut.matches(DayOfWeek::Tue));
        }

        #[test]
        fn test_fwd_delta() {
            let sut = WeekSlot::Range {
                start: DayOfWeek::Wed,
                stop: DayOfWeek::Fri,
            };
            assert_eq!(2, sut.fwd_delta(DayOfWeek::Mon));
            assert_eq!(0, sut.fwd_delta(DayOfWeek::Thu));
            assert_eq!(4, sut.fwd_delta(DayOfWeek::Sat));
        }

        #[test]
        fn test_bwd_delta() {
            let sut = WeekSlot::Range {
                start: DayOfWeek::Wed,
                stop: DayOfWeek::Fri,
            };
            assert_eq!(5, sut.bwd_delta(DayOfWeek::Mon));
            assert_eq!(1, sut.bwd_delta(DayOfWeek::Thu));
            assert_eq!(3, sut.bwd_delta(DayOfWeek::Sat));
        }

        #[test]
        fn test_backward_delta_chrono() {
            let sut = WeekSlot::Range {
                start: DayOfWeek::Wed,
                stop: DayOfWeek::Fri,
            };

            // Inside range (Friday) - snap back to Wednesday start = 2 days
            let input = d(2025, 10, 24, 14, 0, 0); // Friday
            assert_eq!(TimeDelta::days(2), sut.backward_delta_chrono(input));

            // Inside range (Wednesday) - snap back to Wednesday start = 0 days
            let input = d(2025, 10, 22, 14, 0, 0); // Wednesday
            assert_eq!(TimeDelta::days(0), sut.backward_delta_chrono(input));

            // Before range (Monday) - go back to previous week's Wednesday = 2 days
            let input = d(2025, 10, 20, 14, 0, 0); // Monday
            assert_eq!(TimeDelta::days(2), sut.backward_delta_chrono(input));

            // After range (Saturday) - go back to Wednesday (slot start) = 3 days
            let input = d(2025, 10, 25, 14, 0, 0); // Saturday
            assert_eq!(TimeDelta::days(3), sut.backward_delta_chrono(input));
        }

        #[test]
        fn test_backward_delta_chrono_wrap_around() {
            // Range wraps around weekend: Fri-Mon
            let sut = WeekSlot::Range {
                start: DayOfWeek::Fri,
                stop: DayOfWeek::Mon,
            };

            // Inside range (Saturday) - snap back to Friday = 1 day
            let input = d(2025, 10, 25, 14, 0, 0); // Saturday
            assert_eq!(TimeDelta::days(1), sut.backward_delta_chrono(input));

            // Inside range (Monday) - snap back to Friday = 3 days
            let input = d(2025, 10, 20, 14, 0, 0); // Monday
            assert_eq!(TimeDelta::days(3), sut.backward_delta_chrono(input));

            // Outside range (Tuesday) - go back to Friday = 4 days
            let input = d(2025, 10, 21, 14, 0, 0); // Tuesday
            assert_eq!(TimeDelta::days(4), sut.backward_delta_chrono(input));

            // Outside range (Wednesday) - go back to Friday = 5 days
            let input = d(2025, 10, 22, 14, 0, 0); // Wednesday
            assert_eq!(TimeDelta::days(5), sut.backward_delta_chrono(input));
        }
    }
}
