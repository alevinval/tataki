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

    use super::*;
    use crate::types::TimeUnit;

    #[test]
    fn display() {
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
}
