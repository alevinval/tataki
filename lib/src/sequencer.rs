use chrono::DateTime;
use chrono::Local;

use crate::types::Action;
use crate::types::Availability;
use crate::types::Blueprint;
use crate::types::Journal;
use crate::types::Recurrence;

/// Sequences timestamps that satisfy a [Recurrence] within an
/// [Availability].
pub struct Sequencer {
    availability: Availability,
    recurrence: Recurrence,
    remaining: Option<usize>,
    next_minimum_ts: Option<DateTime<Local>>,
}

impl Sequencer {
    pub fn new(
        recurrence: Recurrence,
        availability: Availability,
        last_committed_at: Option<DateTime<Local>>,
    ) -> Self {
        Self {
            availability,
            recurrence,
            remaining: recurrence.remaining(),
            next_minimum_ts: last_committed_at
                .map(|ts| Self::next_minimum_after(recurrence, availability, ts)),
        }
    }

    pub fn from(blueprint: &Blueprint, journal: &Journal) -> Self {
        Self::new(
            blueprint.recurrence(),
            blueprint.availability(),
            journal.last_commit_for(blueprint.id()).and_then(|commit| {
                matches!(commit.action(), Action::Completed).then_some(commit.committed_at())
            }),
        )
    }

    /// Returns true if `ts` is a valid next timestamp in the sequence.
    pub fn accepts(&self, ts: DateTime<Local>) -> bool {
        if let Some(0) = self.remaining {
            return false;
        }

        if let Some(next) = self.next_minimum_ts
            && ts < next
        {
            return false;
        }

        if !self.availability.contains(ts) {
            return false;
        }

        true
    }

    /// Records `ts` as the next occurrence in the sequence.
    pub fn commit(&mut self, ts: DateTime<Local>) {
        debug_assert!(self.accepts(ts), "always guard with `accepts(ts)`");

        if let Some(ref mut r) = self.remaining {
            *r = r.saturating_sub(1);
        }

        self.next_minimum_ts = Some(Self::next_minimum_after(
            self.recurrence,
            self.availability,
            ts,
        ));
    }

    /// Returns the earliest candidate timestamp at or after `ts`.
    pub fn next_candidate_for(&self, ts: DateTime<Local>) -> Option<DateTime<Local>> {
        if let Some(0) = self.remaining {
            return None;
        }

        let earliest = self.next_minimum_ts.map_or(ts, |next| next.max(ts));
        Some(self.availability.next_window_start(earliest))
    }

    fn next_minimum_after(
        recurrence: Recurrence,
        availability: Availability,
        ts: DateTime<Local>,
    ) -> DateTime<Local> {
        let next = ts + recurrence.every().timedelta();
        let inside_window = availability.contains(ts);

        if inside_window && availability.window_end_after(ts).is_none() {
            next
        } else {
            next - availability.backward_delta_chrono(ts)
        }
    }
}

#[cfg(test)]
mod test {

    use super::*;
    use crate::test::d;
    use crate::types::Availability;
    use crate::types::Duration;
    use crate::types::HourSlot;
    use crate::types::WeekSlot;

    #[test]
    fn test_accepts() {
        let sut = Sequencer::new(
            Recurrence::Times {
                count: 3,
                every: Duration::hours(4),
            },
            Availability::new(WeekSlot::full(), HourSlot::fixed(3)),
            None,
        );

        let ts = d(2025, 10, 23, 14, 0, 0);
        assert!(!sut.accepts(ts));

        let ts = d(2025, 10, 23, 3, 0, 0);
        assert!(sut.accepts(ts));

        let ts = d(2025, 10, 24, 3, 0, 0);
        assert!(sut.accepts(ts));
    }

    #[test]
    fn test_commit() {
        let mut sut = Sequencer::new(
            Recurrence::Times {
                count: 2,
                every: Duration::days(2),
            },
            Availability::new(WeekSlot::full(), HourSlot::range(3, 5)),
            None,
        );

        // Outside availability.
        let ts = d(2025, 10, 23, 14, 0, 0);
        assert!(!sut.accepts(ts));

        // Inside availability. Consume.
        let ts = d(2025, 10, 23, 4, 0, 0);
        assert!(sut.accepts(ts));
        sut.commit(ts);
        assert!(!sut.accepts(ts));

        // Inside availability, but not spaced enough.
        let ts = d(2025, 10, 24, 4, 0, 0);
        assert!(!sut.accepts(ts));

        // Inside availability, properly spaced. Consume.
        let ts = d(2025, 10, 25, 4, 0, 0);
        assert!(sut.accepts(ts));
        sut.commit(ts);

        // Inside availability, properly spaced, but no more recurrences available.
        let ts = d(2025, 10, 27, 4, 0, 0);
        assert!(!sut.accepts(ts));
    }

    #[test]
    fn test_next_minimum_ts_keeps_exact_spacing_for_continuous_availability() {
        let availability = Availability::full_week_all_day();
        let recurrence = Recurrence::Period {
            every: Duration::hours(1),
        };

        let ts = d(2026, 6, 15, 1, 30, 0);
        let sut = Sequencer::new(recurrence, availability, Some(ts));
        assert_eq!(sut.next_minimum_ts, Some(d(2026, 6, 15, 2, 30, 0)));
    }

    #[test]
    fn test_next_minimum_ts_snaps_to_next_window_start_after_window_boundary() {
        let availability = Availability::new(WeekSlot::full(), HourSlot::fixed(8));
        let recurrence = Recurrence::Period {
            every: Duration::hours(6),
        };

        let ts_0800 = d(2026, 1, 1, 8, 0, 0);
        let sut = Sequencer::new(recurrence, availability, Some(ts_0800));
        assert_eq!(sut.next_minimum_ts, Some(d(2026, 1, 1, 14, 0, 0)));

        let ts_0900 = d(2026, 1, 1, 9, 0, 0);
        let sut2 = Sequencer::new(recurrence, availability, Some(ts_0900));
        assert_eq!(sut2.next_minimum_ts, Some(d(2026, 1, 1, 14, 0, 0)));

        let ts_0930 = d(2026, 1, 1, 9, 30, 0);
        let sut3 = Sequencer::new(recurrence, availability, Some(ts_0930));
        assert_eq!(sut3.next_minimum_ts, Some(d(2026, 1, 1, 14, 0, 0)));
    }

    #[test]
    fn test_accepts_combined_availability() {
        let sut = Sequencer::new(
            Recurrence::Period {
                every: Duration::days(1),
            },
            Availability::workdays(HourSlot::range(8, 12)),
            None,
        );

        assert!(sut.accepts(d(2026, 10, 26, 9, 0, 0)));
        assert!(!sut.accepts(d(2026, 10, 24, 9, 0, 0)));
        assert!(!sut.accepts(d(2026, 10, 26, 13, 0, 0)));
    }

    #[test]
    fn test_next_candidate_at_or_after() {
        let sut = Sequencer::new(
            Recurrence::Period {
                every: Duration::days(1),
            },
            Availability::workdays(HourSlot::range(8, 12)),
            Some(d(2026, 6, 19, 9, 30, 0)),
        );

        assert_eq!(
            Some(d(2026, 6, 22, 8, 0, 0)),
            sut.next_candidate_for(d(2026, 6, 20, 10, 0, 0))
        );
        assert_eq!(
            Some(d(2026, 6, 22, 9, 0, 0)),
            sut.next_candidate_for(d(2026, 6, 22, 9, 0, 0))
        );
        assert_eq!(
            Some(d(2026, 6, 23, 8, 0, 0)),
            sut.next_candidate_for(d(2026, 6, 22, 13, 0, 0))
        );
    }
}
