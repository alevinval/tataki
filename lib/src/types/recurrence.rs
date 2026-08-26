use serde::Deserialize;
use serde::Serialize;

use crate::types::Duration;

/// Recurrence of an event.
#[derive(Debug, PartialEq, Eq, Clone, Copy, Serialize, Deserialize)]
pub enum Recurrence {
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
    /// Creates a recurrence that occurs exactly once.
    ///
    /// Equivalent to [`Recurrence::Times`] with a count of 1 and zero spacing.
    pub const fn once() -> Self {
        Recurrence::Times {
            count: 1,
            every: Duration::zero(),
        }
    }

    /// Returns the number of remaining occurrences.
    ///
    /// Returns `Some(n)` for a finite number, or `None` for infinite
    /// repetitions.
    pub const fn remaining(self) -> Option<usize> {
        match self {
            Recurrence::Times { count, .. } => Some(count),
            Recurrence::Period { .. } => None,
        }
    }

    /// Returns the spacing between occurrences.
    pub const fn every(self) -> Duration {
        match self {
            Recurrence::Times { every, .. } | Recurrence::Period { every } => every,
        }
    }
}

impl std::fmt::Display for Recurrence {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let args = match self {
            Recurrence::Times { count: 1, every } if *every == Duration::zero() => {
                format_args!("^1")
            }
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

    use super::*;
    use crate::types::TimeUnit;

    #[test]
    fn test_display() {
        let sut = Recurrence::once();
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
    fn test_once() {
        let sut = Recurrence::once();
        assert_eq!(
            sut,
            Recurrence::Times {
                count: 1,
                every: Duration::zero(),
            }
        );
        assert_eq!(Some(1), sut.remaining());
    }

    #[test]
    fn test_remaining() {
        let sut = Recurrence::once();
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
            Recurrence::once(),
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
    fn test_every() {
        let sut = Recurrence::once();
        assert_eq!(Duration::zero(), sut.every());

        let sut = Recurrence::Period {
            every: Duration::days(1),
        };
        assert_eq!(Duration::days(1), sut.every());

        let sut = Recurrence::Times {
            count: 7,
            every: Duration::days(3),
        };
        assert_eq!(Duration::days(3), sut.every());
    }
}
