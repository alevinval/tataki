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
        }
    }
}
