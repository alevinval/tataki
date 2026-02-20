use chrono::DateTime;
use chrono::Local;

use crate::types::Blueprint;
use crate::types::Duration;
use crate::types::Recurrence;
use crate::types::Slot;
use crate::types::experimental::plan_entry::PlannedEntry;

pub struct Generator {
    id: String,
    slot: Slot,
    duration: Duration,
    recurrence: Recurrence,
    remaining: Option<usize>,
    next_mininum_ts: Option<DateTime<Local>>,
}

impl Generator {
    pub fn new(id: String, duration: Duration, recurrence: Recurrence, slot: Slot) -> Self {
        Self {
            id,
            duration,
            slot,
            recurrence,
            remaining: recurrence.remaining(),
            next_mininum_ts: None,
        }
    }

    pub fn from_blueprint(blueprint: &Blueprint) -> Self {
        Self::new(
            blueprint.id().to_string(),
            blueprint.estimated_duration(),
            blueprint.recurrence(),
            blueprint.preferred_slot(),
        )
    }

    /// Returns if [`Self::next`] would generate a [`PlannedEntry`] for the
    /// given `ts`.
    pub fn has_next(&self, ts: DateTime<Local>) -> bool {
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

    /// Generates the next event and consumes one unit of budget when
    /// applicable. Repeated calls to `next(..)` always generate
    /// [`PlannedEntry`] that move forward in time.
    pub fn next(&mut self, ts: DateTime<Local>) -> Option<PlannedEntry> {
        if !self.has_next(ts) {
            return None;
        }

        if let Some(ref mut r) = self.remaining {
            *r = r.saturating_sub(1);
        }

        self.next_mininum_ts = Some(self.recurrence.spaced(ts));

        Some(PlannedEntry::new(self.id.clone(), self.duration, ts))
    }
}

#[cfg(test)]
mod test {

    use chrono::TimeZone;

    use super::*;
    use crate::types::HourSlot;

    #[test]
    fn test_has_next() {
        let sut = Generator::new(
            "id".to_string(),
            Duration::hours(1),
            Recurrence::Times {
                count: 3,
                spacing: Duration::hours(4),
            },
            Slot::Hour(HourSlot::Fixed { hour: 3 }),
        );

        let ts = Local.with_ymd_and_hms(2025, 10, 23, 14, 0, 0).unwrap();
        assert!(!sut.has_next(ts));

        let ts = Local.with_ymd_and_hms(2025, 10, 23, 3, 0, 0).unwrap();
        assert!(sut.has_next(ts));

        let ts = Local.with_ymd_and_hms(2025, 10, 24, 3, 0, 0).unwrap();
        assert!(sut.has_next(ts));
    }

    #[test]
    fn test_next_sequencing() {
        let mut sut = Generator::new(
            "id".to_string(),
            Duration::hours(1),
            Recurrence::Times {
                count: 3,
                spacing: Duration::days(2),
            },
            Slot::Hour(HourSlot::Range { start: 3, stop: 5 }),
        );

        // Outside the slot.
        let ts = Local.with_ymd_and_hms(2025, 10, 23, 14, 0, 0).unwrap();
        assert_eq!(None, sut.next(ts));

        // Inside the slot.
        let ts = Local.with_ymd_and_hms(2025, 10, 23, 4, 0, 0).unwrap();
        assert_eq!(
            Some(PlannedEntry::new("id".to_string(), Duration::hours(1), ts)),
            sut.next(ts)
        );

        // Same slot, but has already planned entry on this slot.
        let ts = Local.with_ymd_and_hms(2025, 10, 23, 4, 0, 0).unwrap();
        assert_eq!(None, sut.next(ts));

        // Next slot, but not spaced enough.
        let ts = Local.with_ymd_and_hms(2025, 10, 24, 4, 0, 0).unwrap();
        assert_eq!(None, sut.next(ts));

        // Next slot, properly spaced.
        let ts = Local.with_ymd_and_hms(2025, 10, 25, 4, 0, 0).unwrap();
        assert_eq!(
            Some(PlannedEntry::new("id".to_string(), Duration::hours(1), ts)),
            sut.next(ts)
        );
    }
}
