use chrono::DateTime;
use chrono::Local;

use crate::types::Blueprint;
use crate::types::Recurrence;
use crate::types::Slot;
use crate::types::experimental::journal::Action;
use crate::types::experimental::journal::Journal;

/// Sequences timestamps that match a [Recurrence] pattern within a [Slot].
///
/// Think of it as an iterator that:
/// - Validates incoming timestamps against a slot (e.g., "every hour at minute
///   0")
/// - Enforces spacing between accepted timestamps (from recurrence)
/// - Tracks remaining count (stops after N occurrences)
pub struct Sequencer {
    slot: Slot,
    recurrence: Recurrence,
    remaining: Option<usize>,
    next_mininum_ts: Option<DateTime<Local>>,
}

impl Sequencer {
    pub fn new(
        recurrence: Recurrence,
        slot: Slot,
        last_committed_at: Option<DateTime<Local>>,
    ) -> Self {
        Self {
            slot,
            recurrence,
            remaining: recurrence.remaining(),
            next_mininum_ts: last_committed_at.map(|ts| recurrence.spaced(ts)),
        }
    }

    pub fn from(blueprint: &Blueprint, journal: &Journal) -> Self {
        Self::new(
            blueprint.recurrence(),
            blueprint.preferred_slot(),
            journal
                .get_last_commit_for(blueprint.id())
                .and_then(|commit| match commit.action() {
                    Action::Completed => Some(commit.committed_at()),
                    Action::Postponed => None,
                }),
        )
    }

    /// Returns true if `ts` is a valid next timestamp in the sequence.
    pub fn accepts(&self, ts: DateTime<Local>) -> bool {
        if let Some(0) = self.remaining {
            return false;
        }

        if let Some(next) = self.next_mininum_ts
            && ts < next
        {
            return false;
        }

        if !self.slot.matches_chrono(ts) {
            return false;
        }

        true
    }

    /// Records `ts` as the next occurrence in the sequence.
    pub fn commit(&mut self, ts: DateTime<Local>) {
        debug_assert!(
            self.accepts(ts),
            "always guard `next()` calls with `has_next()`"
        );

        if let Some(ref mut r) = self.remaining {
            *r = r.saturating_sub(1);
        }

        self.next_mininum_ts = Some(self.recurrence.spaced(ts));
    }
}

#[cfg(test)]
mod test {

    use chrono::TimeZone;

    use super::*;
    use crate::types::Duration;
    use crate::types::HourSlot;

    #[test]
    fn test_accepts() {
        let sut = Sequencer::new(
            Recurrence::Times {
                count: 3,
                spacing: Duration::hours(4),
            },
            Slot::Hour(HourSlot::Fixed { hour: 3 }),
            None,
        );

        let ts = Local.with_ymd_and_hms(2025, 10, 23, 14, 0, 0).unwrap();
        assert!(!sut.accepts(ts));

        let ts = Local.with_ymd_and_hms(2025, 10, 23, 3, 0, 0).unwrap();
        assert!(sut.accepts(ts));

        let ts = Local.with_ymd_and_hms(2025, 10, 24, 3, 0, 0).unwrap();
        assert!(sut.accepts(ts));
    }

    #[test]
    fn test_commit() {
        let mut sut = Sequencer::new(
            Recurrence::Times {
                count: 2,
                spacing: Duration::days(2),
            },
            Slot::Hour(HourSlot::Range { start: 3, stop: 5 }),
            None,
        );

        // Outside slot.
        let ts = Local.with_ymd_and_hms(2025, 10, 23, 14, 0, 0).unwrap();
        assert!(!sut.accepts(ts));

        // Inside slot. Consume.
        let ts = Local.with_ymd_and_hms(2025, 10, 23, 4, 0, 0).unwrap();
        assert!(sut.accepts(ts));
        sut.commit(ts);
        assert!(!sut.accepts(ts));

        // Inside slot, but not spaced enough.
        let ts = Local.with_ymd_and_hms(2025, 10, 24, 4, 0, 0).unwrap();
        assert!(!sut.accepts(ts));

        // Inside slot, properly spaced. Consume.
        let ts = Local.with_ymd_and_hms(2025, 10, 25, 4, 0, 0).unwrap();
        assert!(sut.accepts(ts));
        sut.commit(ts);

        // Inside slot, properly spaced, but no more recurrences available.
        let ts = Local.with_ymd_and_hms(2025, 10, 27, 4, 0, 0).unwrap();
        assert!(!sut.accepts(ts));
    }
}
