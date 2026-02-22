pub use hour_slots::HourSlot;

mod hour_slots;

use chrono::DateTime;
use chrono::TimeZone;
use chrono::Timelike;

/// A time slot for scheduling affinity.
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum Slot {
    /// A specific hour or hour range in a day (0-23).
    Hour(HourSlot),
}

impl Slot {
    /// Returns if the timestamp matches the affinity represented by the slot.
    fn matches<T: TimeZone>(&self, ts: DateTime<T>) -> bool {
        match self {
            Slot::Hour(hour_slot) => hour_slot.matches(ts.hour() as u8),
        }
    }
}

impl std::fmt::Display for Slot {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Slot::Hour(hour_slot) => f.write_fmt(format_args!("{}", hour_slot)),
        }
    }
}
