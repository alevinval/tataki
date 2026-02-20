pub use hour_slots::HourSlot;
pub use week_slots::WeekSlot;

mod hour_slots;
mod week_slots;

use chrono::DateTime;
use chrono::TimeDelta;
use chrono::TimeZone;

/// A time slot for scheduling affinity.
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
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
            Slot::Hour(hour_slot) => hour_slot.matches_chrono(ts),
            Slot::Week(week_slot) => week_slot.matches_chrono(ts),
        }
    }

    /// Returns the [`TimeDelta`] that `ts` must advance to fit within the slot.
    pub fn fwd_delta_chrono<T: TimeZone>(&self, ts: DateTime<T>) -> TimeDelta {
        match self {
            Slot::Hour(hour_slot) => hour_slot.fwd_delta_chrono(ts),
            Slot::Week(week_slot) => week_slot.fwd_delta_chrono(ts),
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
