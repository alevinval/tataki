use chrono::DateTime;
use chrono::Local;
use serde::Deserialize;
use serde::Serialize;

use crate::types::Duration;

/// Recurrence of an event.
#[derive(Debug, PartialEq, Eq, Clone, Copy, Serialize, Deserialize)]
pub enum Recurrence {
    /// Occurs exactly once, does not repeat.
    Once,

    /// Repeats a fixed number of times at regular intervals.
    ///
    /// The event occurs `count` times, with each occurrence spaced
    /// by `every` duration. Stops automatically after the final occurrence.
    Times { count: usize, every: Duration },

    /// Repeats indefinitely at regular intervals.
    ///
    /// The event repeats forever, with each occurrence spaced by
    /// `every` duration. Does not stop unless explicitly cancelled.
    Period { every: Duration },
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
            Recurrence::Times { every, .. } | Recurrence::Period { every } => {
                ts + every.timedelta()
            }
        }
    }
}

impl std::fmt::Display for Recurrence {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let args = match self {
            Recurrence::Once => format_args!("^1"),
            Recurrence::Times { count, every } => {
                format_args!("^{{{},{}}}", *count, *every)
            }
            Recurrence::Period { every: spacing } => format_args!("^{}", *spacing),
        };
        f.write_fmt(args)
    }
}

#[cfg(test)]
mod test {

    use chrono::TimeDelta;

    use super::*;
    use crate::test::d;
    use crate::types::TimeUnit;

    #[test]
    fn test_display() {
        let sut = Recurrence::Once;
        assert_eq!("^1", sut.to_string());

        let sut = Recurrence::Times {
            count: 3,
            every: Duration::days(2),
        };
        assert_eq!("^{3,2d}", sut.to_string());

        let sut = Recurrence::Period {
            every: Duration::of(3, TimeUnit::Year),
        };
        assert_eq!("^3y", sut.to_string());
    }

    #[test]
    fn test_remaining() {
        let sut = Recurrence::Once;
        assert_eq!(Some(1), sut.remaining());

        let sut = Recurrence::Period {
            every: Duration::days(1),
        };
        assert_eq!(None, sut.remaining());

        let sut = Recurrence::Times {
            count: 7,
            every: Duration::days(1),
        };
        assert_eq!(Some(7), sut.remaining());
    }

    #[test]
    fn test_serde_roundtrip() {
        let suts = [
            Recurrence::Once,
            Recurrence::Times {
                count: 3,
                every: Duration::days(2),
            },
            Recurrence::Period {
                every: Duration::of(3, TimeUnit::Month),
            },
        ];
        for sut in suts {
            let json = serde_json::to_string(&sut).unwrap();
            let back: Recurrence = serde_json::from_str(&json).unwrap();
            assert_eq!(sut, back);
        }
    }

    #[test]
    fn test_spaced() {
        let ts = d(2026, 10, 23, 0, 0, 0);

        let sut = Recurrence::Once;
        assert_eq!(ts, sut.spaced(ts));

        let sut = Recurrence::Period {
            every: Duration::days(1),
        };
        assert_eq!(ts + TimeDelta::days(1), sut.spaced(ts));

        let sut = Recurrence::Times {
            count: 7,
            every: Duration::days(3),
        };
        assert_eq!(ts + TimeDelta::days(3), sut.spaced(ts));
    }
}
