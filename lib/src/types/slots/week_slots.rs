use chrono::DateTime;
use chrono::Datelike;
use chrono::TimeDelta;
use chrono::TimeZone;
use serde::Deserialize;
use serde::Serialize;

use crate::types::days::DayOfWeek;

#[derive(Debug, PartialEq, Eq, Clone, Copy, Serialize, Deserialize)]
pub enum WeekSlot {
    /// A specific day of the week.
    Fixed { day: DayOfWeek },
    /// An inclusive range of days `[start, stop]`
    Range { start: DayOfWeek, stop: DayOfWeek },
}

impl WeekSlot {
    pub const fn workdays() -> Self {
        Self::Range {
            start: DayOfWeek::Mon,
            stop: DayOfWeek::Fri,
        }
    }

    pub const fn weekend() -> Self {
        Self::Range {
            start: DayOfWeek::Sat,
            stop: DayOfWeek::Sun,
        }
    }

    pub const fn full() -> Self {
        Self::Range {
            start: DayOfWeek::Mon,
            stop: DayOfWeek::Sun,
        }
    }

    pub fn matches(&self, day: DayOfWeek) -> bool {
        match self {
            Self::Fixed { day: d } => *d == day,
            Self::Range { start, stop } if start <= stop => (*start..=*stop).contains(&day),
            Self::Range { start, stop } => day >= *start || day <= *stop,
        }
    }

    pub fn forward_delta(&self, curr: DayOfWeek) -> i64 {
        if self.matches(curr) {
            0
        } else {
            (self.start_day() as i64 - curr as i64 + 7) % 7
        }
    }

    pub fn backward_delta(&self, curr: DayOfWeek) -> i64 {
        let start = self.start_day() as i64;
        let curr = curr as i64;

        match self {
            Self::Fixed { .. } => (start - curr + 7) % 7,
            Self::Range {
                start: range_start,
                stop,
            } if range_start <= stop => {
                if curr < start {
                    start - curr
                } else {
                    curr - start
                }
            }
            Self::Range { .. } => (curr - start + 7) % 7,
        }
    }

    pub fn matches_chrono<T: TimeZone>(&self, ts: DateTime<T>) -> bool {
        self.matches(ts.weekday().into())
    }

    pub fn forward_delta_chrono<T: TimeZone>(&self, ts: DateTime<T>) -> TimeDelta {
        TimeDelta::days(self.forward_delta(ts.weekday().into()))
    }

    pub fn backward_delta_chrono<T: TimeZone>(&self, ts: DateTime<T>) -> TimeDelta {
        TimeDelta::days(self.backward_delta(ts.weekday().into()))
    }

    const fn start_day(&self) -> DayOfWeek {
        match self {
            Self::Fixed { day } | Self::Range { start: day, .. } => *day,
        }
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
        fn test_forward_delta() {
            let sut = WeekSlot::Fixed {
                day: DayOfWeek::Mon,
            };
            assert_eq!(4, sut.forward_delta(DayOfWeek::Thu));

            let sut = WeekSlot::Fixed {
                day: DayOfWeek::Tue,
            };
            assert_eq!(0, sut.forward_delta(DayOfWeek::Tue));

            let sut = WeekSlot::Fixed {
                day: DayOfWeek::Wed,
            };
            assert_eq!(5, sut.forward_delta(DayOfWeek::Fri));
        }

        #[test]
        fn test_backward_delta() {
            let sut = WeekSlot::Fixed {
                day: DayOfWeek::Mon,
            };
            assert_eq!(4, sut.backward_delta(DayOfWeek::Thu));

            let sut = WeekSlot::Fixed {
                day: DayOfWeek::Tue,
            };
            assert_eq!(0, sut.backward_delta(DayOfWeek::Tue));

            let sut = WeekSlot::Fixed {
                day: DayOfWeek::Wed,
            };
            assert_eq!(5, sut.backward_delta(DayOfWeek::Fri));
        }

        #[test]
        fn test_chrono_interop() {
            let sut = WeekSlot::Fixed {
                day: DayOfWeek::Tue,
            };
            let input = d(2025, 10, 23, 14, 0, 0);

            assert_eq!(TimeDelta::days(5), sut.forward_delta_chrono(input));

            assert!(sut.matches_chrono(input - TimeDelta::days(2)));

            assert!(!sut.matches_chrono(input));
        }

        #[test]
        fn test_backward_delta_chrono() {
            let sut = WeekSlot::Fixed {
                day: DayOfWeek::Fri,
            };

            // Inside range (Friday) - no backward delta needed
            let input = d(2025, 10, 24, 14, 0, 0); // Friday
            assert_eq!(TimeDelta::days(0), sut.backward_delta_chrono(input));

            // Outside range - go back to last Friday = 1 day
            let input = d(2025, 10, 23, 14, 0, 0); // Thursday
            assert_eq!(TimeDelta::days(1), sut.backward_delta_chrono(input));

            // Outside range - go back to last Friday = 3 days
            let input = d(2025, 10, 21, 14, 0, 0); // Tuesday
            assert_eq!(TimeDelta::days(3), sut.backward_delta_chrono(input));
        }

        #[test]
        fn test_serde_roundtrip() {
            let suts = [
                WeekSlot::Fixed {
                    day: DayOfWeek::Mon,
                },
                WeekSlot::Fixed {
                    day: DayOfWeek::Sun,
                },
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
        fn test_forward_delta() {
            let sut = WeekSlot::Range {
                start: DayOfWeek::Wed,
                stop: DayOfWeek::Fri,
            };
            assert_eq!(2, sut.forward_delta(DayOfWeek::Mon));
            assert_eq!(0, sut.forward_delta(DayOfWeek::Thu));
            assert_eq!(4, sut.forward_delta(DayOfWeek::Sat));
        }

        #[test]
        fn test_backward_delta() {
            let sut = WeekSlot::Range {
                start: DayOfWeek::Wed,
                stop: DayOfWeek::Fri,
            };
            assert_eq!(2, sut.backward_delta(DayOfWeek::Mon));
            assert_eq!(1, sut.backward_delta(DayOfWeek::Thu));
            assert_eq!(3, sut.backward_delta(DayOfWeek::Sat));
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

            // After range (Saturday) - go back to Wednesday (range start) = 3 days
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

        #[test]
        fn test_serde_roundtrip() {
            let suts = [
                WeekSlot::Range {
                    start: DayOfWeek::Mon,
                    stop: DayOfWeek::Fri,
                },
                WeekSlot::Range {
                    start: DayOfWeek::Fri,
                    stop: DayOfWeek::Mon,
                },
            ];
            for sut in suts {
                let json = serde_json::to_string(&sut).unwrap();
                let back: WeekSlot = serde_json::from_str(&json).unwrap();
                assert_eq!(sut, back);
            }
        }
    }
}
