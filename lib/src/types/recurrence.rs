use chrono::DateTime;
use chrono::Local;

use crate::types::Duration;

/// Recurrence of an event.
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum Recurrence {
    /// Occurs exactly once, does not repeat.
    Once,

    /// Repeats a fixed number of times at regular intervals.
    ///
    /// The event occurs `count` times, with each occurrence spaced
    /// by `every` duration. Stops automatically after the final occurrence.
    Times { count: usize, spacing: Duration },

    /// Repeats indefinitely at regular intervals.
    ///
    /// The event repeats forever, with each occurrence spaced by
    /// `spacing` duration. Does not stop unless explicitly cancelled.
    Period { spacing: Duration },
}

impl Recurrence {
    /// Returns the number of remaining occurrences.
    ///
    /// Returns `Some(n)` for a finite number, or `None` for infinite
    /// repetitions.
    pub const fn remaining(self) -> Option<usize> {
        match self {
            Recurrence::Once => Some(1),
            Recurrence::Times { count, .. } => Some(count),
            Recurrence::Period { .. } => None,
        }
    }

    /// Returns a `ts` with the spacing of the recurrence applied.
    pub fn spaced(self, ts: DateTime<Local>) -> DateTime<Local> {
        match self {
            Recurrence::Once => ts,
            Recurrence::Times { spacing, .. } | Recurrence::Period { spacing } => {
                ts + spacing.timedelta()
            }
        }
    }
}

impl std::fmt::Display for Recurrence {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let args = match self {
            Recurrence::Once => format_args!("^1"),
            Recurrence::Times { count, spacing } => {
                format_args!("^{{{},{}}}", *count, *spacing)
            }
            Recurrence::Period { spacing } => format_args!("^{}", *spacing),
        };
        f.write_fmt(args)
    }
}

#[cfg(test)]
mod test {

    use chrono::TimeDelta;
    use chrono::TimeZone;

    use super::*;
    use crate::types::TimeUnit;

    #[test]
    fn test_display() {
        let sut = Recurrence::Once;
        assert_eq!("^1", sut.to_string());

        let sut = Recurrence::Times {
            count: 3,
            spacing: Duration::days(2),
        };
        assert_eq!("^{3,2d}", sut.to_string());

        let sut = Recurrence::Period {
            spacing: Duration::of(3, TimeUnit::Year),
        };
        assert_eq!("^3y", sut.to_string());
    }

    #[test]
    fn test_remaining() {
        let sut = Recurrence::Once;
        assert_eq!(Some(1), sut.remaining());

        let sut = Recurrence::Period {
            spacing: Duration::days(1),
        };
        assert_eq!(None, sut.remaining());

        let sut = Recurrence::Times {
            count: 7,
            spacing: Duration::days(1),
        };
        assert_eq!(Some(7), sut.remaining());
    }

    #[test]
    fn test_spaced() {
        let ts = Local.with_ymd_and_hms(2026, 10, 23, 0, 0, 0).unwrap();

        let sut = Recurrence::Once;
        assert_eq!(ts, sut.spaced(ts));

        let sut = Recurrence::Period {
            spacing: Duration::days(1),
        };
        assert_eq!(ts + TimeDelta::days(1), sut.spaced(ts));

        let sut = Recurrence::Times {
            count: 7,
            spacing: Duration::days(3),
        };
        assert_eq!(ts + TimeDelta::days(3), sut.spaced(ts));
    }
}
