pub use hour_slots::HourSlot;
pub use week_slots::WeekSlot;

mod hour_slots;
mod week_slots;

use chrono::DateTime;
use chrono::TimeDelta;
use chrono::TimeZone;
use serde::Deserialize;
use serde::Serialize;

/// A time slot for scheduling affinity.
#[derive(Debug, PartialEq, Eq, Clone, Copy, Serialize, Deserialize)]
pub enum Slot {
    /// A specific hour or hour range in a day (0-23).
    Hour(HourSlot),
    /// A day of the week, or range of days (Mon-Sun).
    Week(WeekSlot),
}

impl Slot {
    /// Returns if `ts` matches the affinity represented by the slot.
    pub fn matches_chrono<T: TimeZone>(&self, ts: DateTime<T>) -> bool {
        match self {
            Slot::Hour(slot) => slot.matches_chrono(ts),
            Slot::Week(slot) => slot.matches_chrono(ts),
        }
    }

    /// Returns the [`TimeDelta`] that `ts` must advance to fit within the
    /// initial boundary of the slot.
    pub fn fwd_delta_chrono<T: TimeZone>(&self, ts: DateTime<T>) -> TimeDelta {
        match self {
            Slot::Hour(slot) => slot.fwd_delta_chrono(ts),
            Slot::Week(slot) => slot.fwd_delta_chrono(ts),
        }
    }

    /// Returns the [`TimeDelta`] that `ts` must recede to fit within the
    /// initial boundary of the slot.
    pub fn bwd_delta_chrono<T: TimeZone>(&self, ts: DateTime<T>) -> TimeDelta {
        match self {
            Slot::Hour(slot) => slot.bwd_delta_chrono(ts),
            Slot::Week(slot) => slot.bwd_delta_chrono(ts),
        }
    }

    pub fn backward_delta_chrono<T: TimeZone>(&self, ts: DateTime<T>) -> TimeDelta {
        match self {
            Slot::Hour(hour_slot) => hour_slot.backward_delta_chrono(ts),
            Slot::Week(week_slot) => week_slot.backward_delta_chrono(ts),
        }
    }
}

impl std::fmt::Display for Slot {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Slot::Hour(hour_slot) => f.write_fmt(format_args!("{}", hour_slot)),
            Slot::Week(week_slot) => f.write_fmt(format_args!("{}", week_slot)),
        }
    }
}

#[cfg(test)]
mod test {

    use super::*;
    use crate::types::days::DayOfWeek;

    #[test]
    fn test_serde_roundtrip() {
        let suts = [
            Slot::Hour(HourSlot::Fixed { hour: 9 }),
            Slot::Hour(HourSlot::Range { start: 8, stop: 12 }),
            Slot::Week(WeekSlot::Fixed {
                day: DayOfWeek::Wed,
            }),
            Slot::Week(WeekSlot::Range {
                start: DayOfWeek::Fri,
                stop: DayOfWeek::Mon,
            }),
        ];
        for sut in suts {
            let json = serde_json::to_string(&sut).unwrap();
            let back: Slot = serde_json::from_str(&json).unwrap();
            assert_eq!(sut, back);
        }
    }
}
