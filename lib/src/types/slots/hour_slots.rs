/// Represents a specific hour or hour range in a day (0-23).
///
/// Used to specify when a [`Blueprint`](crate::types::Blueprint) has
/// affinity and should be materialized on particular hours of the day.
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum HourSlot {
    /// A specific hour of the day.
    Fixed { hour: u8 },

    /// An inclusive range of hours `[start, end]`
    Range { start: u8, end: u8 },
}

impl HourSlot {
    /// Returns true if the given hour (0-23) falls within this slot.
    pub fn matches(&self, hour: u8) -> bool {
        debug_assert!(hour < 24, "hour must be <24, instead it was {hour}");

        match self {
            HourSlot::Fixed { hour: h } => *h == hour,
            HourSlot::Range { start, end } => {
                if start < end {
                    (*start..=*end).contains(&hour)
                } else {
                    hour >= *start || hour <= *end
                }
            }
        }
    }
}

impl std::fmt::Display for HourSlot {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            HourSlot::Fixed { hour } => write!(f, "{:02}:00", hour),
            HourSlot::Range { start, end } => write!(f, "{:02}:00-{:02}:00", start, end),
        }
    }
}

#[cfg(test)]
mod test {

    use super::*;

    mod slot_hour {
        use super::*;

        mod fixed {
            use super::*;

            #[test]
            fn test_matches() {
                let sut = HourSlot::Fixed { hour: 12 };
                assert!(sut.matches(12));
                assert!(!sut.matches(11));
                assert!(!sut.matches(13));
            }
        }

        mod range {
            use super::*;

            #[test]
            fn test_matches() {
                let sut = HourSlot::Range { start: 8, end: 3 };
                assert!(sut.matches(8));
                assert!(sut.matches(23));
                assert!(sut.matches(0));
                assert!(sut.matches(3));

                assert!(!sut.matches(4));
                assert!(!sut.matches(7));
            }
        }
    }
}
