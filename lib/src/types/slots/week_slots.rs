use chrono::DateTime;
use chrono::Datelike;
use chrono::TimeDelta;
use chrono::TimeZone;

use crate::types::days::DayOfWeek;

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum WeekSlot {
    /// A specific day of the week.
    Fixed { day: DayOfWeek },
    /// An inclusive range of days.
    Range { from: DayOfWeek, to: DayOfWeek },
}

impl WeekSlot {
    /// Returns a range that covers all working days.
    pub const fn workdays() -> Self {
        Self::Range {
            from: DayOfWeek::Mon,
            to: DayOfWeek::Fri,
        }
    }

    /// Returns a range that covers the weekend.
    pub const fn weekend() -> Self {
        Self::Range {
            from: DayOfWeek::Sat,
            to: DayOfWeek::Sun,
        }
    }

    /// Returns a range that spans a full week
    pub const fn full() -> Self {
        Self::Range {
            from: DayOfWeek::Mon,
            to: DayOfWeek::Sun,
        }
    }

    /// Returns true if the given day falls within this slot.
    pub fn matches(&self, day: DayOfWeek) -> bool {
        match self {
            WeekSlot::Fixed { day: d } => *d == day,
            WeekSlot::Range { from, to } => {
                if *from <= *to {
                    (*from..=*to).contains(&day)
                } else {
                    day >= *from || day <= *to
                }
            }
        }
    }

    /// Computes the forward delta in days, if the input does not fit the slot,
    /// rather than returning negative, it computes the delta until the next
    /// week.
    pub const fn fwd_delta(&self, src: DayOfWeek) -> i64 {
        let src = src as u32;
        let pivot = match self {
            Self::Fixed { day } => {
                let day = *day as u32;
                if src <= day { day } else { day + 7 }
            }
            Self::Range {
                from: start,
                to: end,
            } => {
                let (start, end) = (*start as u32, *end as u32);
                if src <= start {
                    start
                } else if src > end {
                    start + 7
                } else {
                    src
                }
            }
        };
        pivot as i64 - src as i64
    }

    pub fn matches_chrono<T: TimeZone>(&self, src: DateTime<T>) -> bool {
        self.matches(src.weekday().into())
    }

    pub fn fwd_delta_chrono<T: TimeZone>(&self, src: DateTime<T>) -> TimeDelta {
        TimeDelta::days(self.fwd_delta(src.weekday().into()))
    }
}

impl std::fmt::Display for WeekSlot {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            WeekSlot::Fixed { day } => write!(f, "{}", day),
            WeekSlot::Range { from, to } => write!(f, "{}-{}", from, to),
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    mod slot_week {
        use super::*;

        mod fixed {

            use chrono::Utc;

            use super::*;

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
            fn test_chrono_interop() {
                let sut = WeekSlot::Fixed {
                    day: DayOfWeek::Tue,
                };
                let input = Utc.with_ymd_and_hms(2025, 10, 23, 14, 0, 0).unwrap();

                assert_eq!(TimeDelta::days(5), sut.fwd_delta_chrono(input));

                assert!(sut.matches_chrono(input - TimeDelta::days(2)));

                assert!(!sut.matches_chrono(input));
            }
        }

        mod range {
            use super::*;

            #[test]
            fn test_matches_wrap_around() {
                let sut = WeekSlot::Range {
                    from: DayOfWeek::Fri,
                    to: DayOfWeek::Mon,
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
                    from: DayOfWeek::Wed,
                    to: DayOfWeek::Fri,
                };
                assert_eq!(2, sut.fwd_delta(DayOfWeek::Mon));
                assert_eq!(0, sut.fwd_delta(DayOfWeek::Thu));
                assert_eq!(4, sut.fwd_delta(DayOfWeek::Sat));
            }
        }
    }
}
