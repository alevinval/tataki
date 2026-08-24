use chrono::DateTime;
use chrono::Local;
use chrono::TimeDelta;

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
    next_minimum_ts: Option<DateTime<Local>>,
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
            next_minimum_ts: last_committed_at
                .map(|ts| recurrence.spaced(ts) - slot.backward_delta_chrono(ts)),
        }
    }

    pub fn from(blueprint: &Blueprint, journal: &Journal) -> Self {
        Self::new(
            blueprint.recurrence(),
            blueprint.preferred_slot(),
            journal
                .last_commit_for(blueprint.id())
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

        if let Some(next) = self.next_minimum_ts
            && ts < next
        {
            return false;
        }

        if !self.slot.matches_chrono(ts) {
            return false;
        }

        true
    }

    /// The smallest delta that advances `ts` past its violated slot or
    /// spacing constraint, if either is violated.
    pub fn fwd_delta(&self, ts: DateTime<Local>) -> Option<TimeDelta> {
        if self.remaining == Some(0) {
            return None;
        }

        let spacing = self
            .next_minimum_ts
            .filter(|next| *next > ts)
            .map(|next| next - ts);
        let slot = self.slot.fwd_delta_chrono(ts);
        let slot = if slot.is_zero() { None } else { Some(slot) };
        [spacing, slot].into_iter().flatten().min()
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

        self.next_minimum_ts =
            Some(self.recurrence.spaced(ts) - self.slot.backward_delta_chrono(ts));
    }
}

#[cfg(test)]
mod test {

    use super::*;
    use crate::test::d;
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
                spacing: Duration::days(2),
            },
            Slot::Hour(HourSlot::Range { start: 3, stop: 5 }),
            None,
        );

        // Outside slot.
        let ts = d(2025, 10, 23, 14, 0, 0);
        assert!(!sut.accepts(ts));

        // Inside slot. Consume.
        let ts = d(2025, 10, 23, 4, 0, 0);
        assert!(sut.accepts(ts));
        sut.commit(ts);
        assert!(!sut.accepts(ts));

        // Inside slot, but not spaced enough.
        let ts = d(2025, 10, 24, 4, 0, 0);
        assert!(!sut.accepts(ts));

        // Inside slot, properly spaced. Consume.
        let ts = d(2025, 10, 25, 4, 0, 0);
        assert!(sut.accepts(ts));
        sut.commit(ts);

        // Inside slot, properly spaced, but no more recurrences available.
        let ts = d(2025, 10, 27, 4, 0, 0);
        assert!(!sut.accepts(ts));
    }

    mod fwd_delta {
        use super::*;

        /// Fixed slot at 03:00, 4h spacing, committed at 2025-10-23 03:00
        /// (→ next_minimum_ts = 07:00).
        fn sut() -> Sequencer {
            let slot = Slot::Hour(HourSlot::Fixed { hour: 3 });
            let recurrence = Recurrence::Times {
                count: 3,
                spacing: Duration::hours(4),
            };
            Sequencer::new(recurrence, slot, Some(d(2025, 10, 23, 3, 0, 0)))
        }

        /// Range slot 03:00-05:00, 2d spacing, committed at 2025-10-23 04:00
        /// (→ next_minimum_ts = 2025-10-25 03:00).
        fn sut_range() -> Sequencer {
            let slot = Slot::Hour(HourSlot::Range { start: 3, stop: 5 });
            let recurrence = Recurrence::Times {
                count: 2,
                spacing: Duration::days(2),
            };
            Sequencer::new(recurrence, slot, Some(d(2025, 10, 23, 4, 0, 0)))
        }

        #[test]
        fn no_committed_ts() {
            let slot = Slot::Hour(HourSlot::Fixed { hour: 3 });
            let recurrence = Recurrence::Times {
                count: 3,
                spacing: Duration::hours(4),
            };
            let sut = Sequencer::new(recurrence, slot, None);
            // Inside slot → no violated constraint.
            assert_eq!(sut.fwd_delta(d(2025, 10, 23, 3, 0, 0)), None);
            // Outside slot → advance to next 03:00.
            assert_eq!(
                sut.fwd_delta(d(2025, 10, 23, 14, 0, 0)),
                Some(TimeDelta::hours(13))
            );
        }

        #[test]
        fn spacing_violation() {
            let sut = sut();
            // Inside slot, before next_minimum_ts → spacing delta.
            assert_eq!(
                sut.fwd_delta(d(2025, 10, 23, 3, 0, 0)),
                Some(TimeDelta::hours(4))
            );
            assert_eq!(
                sut.fwd_delta(d(2025, 10, 23, 4, 0, 0)),
                Some(TimeDelta::hours(3))
            );

            // Wide spacing: inside slot, far before next_minimum_ts.
            let sut = sut_range();
            assert_eq!(
                sut.fwd_delta(d(2025, 10, 24, 4, 0, 0)),
                Some(TimeDelta::hours(23))
            );
        }

        #[test]
        fn slot_violation() {
            let sut = sut();
            // Spacing satisfied, outside slot → slot delta.
            assert_eq!(
                sut.fwd_delta(d(2025, 10, 23, 7, 0, 0)),
                Some(TimeDelta::hours(20))
            );
        }

        #[test]
        fn both_violated_min_wins() {
            let sut = sut_range();
            // Outside slot: slot delta (15h) < spacing delta (39h) → slot delta.
            assert_eq!(
                sut.fwd_delta(d(2025, 10, 23, 12, 0, 0)),
                Some(TimeDelta::hours(15))
            );
        }

        #[test]
        fn satisfied() {
            let sut = sut_range();
            // Spacing satisfied and inside slot → no violated constraint.
            assert_eq!(sut.fwd_delta(d(2025, 10, 25, 4, 0, 0)), None);
        }

        #[test]
        fn exhausted() {
            let mut sut = sut_range();
            sut.commit(d(2025, 10, 25, 4, 0, 0));
            sut.commit(d(2025, 10, 27, 4, 0, 0));
            // No occurrences left → no forward delta.
            assert_eq!(sut.fwd_delta(d(2025, 10, 27, 4, 0, 0)), None);
        }
    }

    #[test]
    fn test_next_minimum_ts_with_backward_delta() {
        let slot = Slot::Hour(HourSlot::Fixed { hour: 8 });
        let recurrence = Recurrence::Period {
            spacing: Duration::hours(6),
        };

        // Case 1: First commit at 08:00
        let ts_0800 = d(2026, 1, 1, 8, 0, 0);
        let sut = Sequencer::new(recurrence, slot, Some(ts_0800));
        // next_minimum_ts = 14:00 - 0 = 14:00
        assert_eq!(sut.next_minimum_ts, Some(d(2026, 1, 1, 14, 0, 0)));

        // Case 2: Scheduler advances to 09:00 and commits there
        let ts_0900 = d(2026, 1, 1, 9, 0, 0);
        let sut2 = Sequencer::new(recurrence, slot, Some(ts_0900));
        // next_minimum_ts = 15:00 - 1h = 14:00
        assert_eq!(sut2.next_minimum_ts, Some(d(2026, 1, 1, 14, 0, 0)));
    }
}
