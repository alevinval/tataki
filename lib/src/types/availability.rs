use chrono::DateTime;
use chrono::Datelike;
use chrono::Days;
use chrono::Local;
use chrono::NaiveTime;
use chrono::TimeDelta;
use chrono::TimeZone;
use chrono::Timelike;
use serde::Deserialize;
use serde::Serialize;

use crate::types::Duration;
use crate::types::HourSlot;
use crate::types::WeekSlot;

#[derive(Debug, PartialEq, Eq, Clone, Copy, Serialize, Deserialize)]
pub struct Availability {
    days: WeekSlot,
    hours: HourSlot,
}

impl Availability {
    const ALL_DAY_HOURS: HourSlot = HourSlot { start: 0, stop: 23 };
    const MAX_DAY_SCAN: u64 = 7;
    const MAX_HOUR_SCAN: i64 = 24 * 8;

    pub const fn new(days: WeekSlot, hours: HourSlot) -> Self {
        Self { days, hours }
    }

    const fn start_hour(&self) -> u32 {
        self.hours.start
    }

    pub const fn workdays(hours: HourSlot) -> Self {
        Self::new(WeekSlot::workdays(), hours)
    }

    pub const fn anytime(days: WeekSlot) -> Self {
        Self::new(days, Self::ALL_DAY_HOURS)
    }

    pub const fn full_week_all_day() -> Self {
        Self::new(WeekSlot::full(), Self::ALL_DAY_HOURS)
    }

    pub const fn days(&self) -> WeekSlot {
        self.days
    }

    pub const fn hours(&self) -> HourSlot {
        self.hours
    }

    pub fn contains<T: TimeZone>(&self, ts: DateTime<T>) -> bool {
        self.days.matches(ts.weekday().into()) && self.hours.matches(ts.hour())
    }

    pub fn backward_delta_chrono(&self, ts: DateTime<Local>) -> TimeDelta {
        ts - self.backwards_window_start(ts)
    }

    /// Returns the next timestamp at or after `ts` that lies inside this
    /// availability window.
    pub fn next_window_start(&self, ts: DateTime<Local>) -> DateTime<Local> {
        if self.contains(ts) {
            return ts;
        }

        (0..=Self::MAX_DAY_SCAN)
            .map(|days_fwd| ts + Days::new(days_fwd))
            .filter(|date| self.days.matches(date.weekday().into()))
            .map(|date| self.starting_hour_for(date))
            .find(|candidate| *candidate >= ts)
            .expect("availability must have a next matching window")
    }

    /// Returns the exclusive end of the current window containing `ts`.
    pub fn window_end_after(&self, ts: DateTime<Local>) -> Option<DateTime<Local>> {
        if !self.contains(ts) {
            return None;
        }

        let mut boundary = self.next_hour_boundary(ts);
        for _ in 0..=Self::MAX_HOUR_SCAN {
            if !self.contains(boundary) {
                return Some(boundary);
            }
            boundary += TimeDelta::hours(1);
        }

        None
    }

    /// Returns whether `duration` fits entirely within the current window
    /// starting at `start`.
    pub fn can_fit(&self, start: DateTime<Local>, duration: Duration) -> bool {
        if !self.contains(start) {
            return false;
        }

        match self.window_end_after(start) {
            Some(end) => start + duration.timedelta() <= end,
            None => true,
        }
    }

    fn backwards_window_start(&self, ts: DateTime<Local>) -> DateTime<Local> {
        (0..=Self::MAX_DAY_SCAN)
            .map(|days_back| ts - Days::new(days_back))
            .filter(|date| self.days.matches(date.weekday().into()))
            .map(|date| self.starting_hour_for(date))
            .find(|candidate| *candidate <= ts)
            .expect("availability must have a previous matching window")
    }

    fn starting_hour_for(&self, date: DateTime<Local>) -> DateTime<Local> {
        Local
            .with_ymd_and_hms(
                date.year(),
                date.month(),
                date.day(),
                self.start_hour(),
                0,
                0,
            )
            .unwrap()
    }

    fn next_hour_boundary(&self, ts: DateTime<Local>) -> DateTime<Local> {
        ts.with_time(NaiveTime::from_hms_nano_opt(ts.hour() + 1, 0, 0, 0).unwrap())
            .unwrap()
    }
}

impl std::fmt::Display for Availability {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let full_week = self.days == WeekSlot::full();
        let all_day = self.hours == Self::ALL_DAY_HOURS;

        match (full_week, all_day) {
            (true, true) => f.write_str("Mon-Sun 00:00-23:00"),
            (true, false) => self.hours.fmt(f),
            (false, true) => self.days.fmt(f),
            (false, false) => write!(f, "{} {}", self.days, self.hours),
        }
    }
}

#[cfg(test)]
mod test {
    use pretty_assertions::assert_eq;

    use super::*;
    use crate::test::d;
    use crate::types::days::DayOfWeek;

    #[test]
    fn test_contains() {
        let sut = Availability::workdays(HourSlot { start: 8, stop: 17 });

        assert!(sut.contains(d(2026, 10, 26, 9, 0, 0)));
        assert!(!sut.contains(d(2026, 10, 24, 9, 0, 0)));
        assert!(!sut.contains(d(2026, 10, 26, 19, 0, 0)));
    }

    #[test]
    fn test_constructors() {
        let sut = Availability::anytime(WeekSlot::Fixed {
            day: DayOfWeek::Wed,
        });
        assert_eq!(
            Availability::new(
                WeekSlot::Fixed {
                    day: DayOfWeek::Wed
                },
                HourSlot { start: 0, stop: 23 }
            ),
            sut
        );

        assert_eq!(
            Availability::new(WeekSlot::full(), HourSlot { start: 0, stop: 23 }),
            Availability::full_week_all_day()
        );
    }

    #[test]
    fn test_backward_delta_chrono() {
        let sut = Availability::workdays(HourSlot { start: 8, stop: 12 });

        assert_eq!(
            TimeDelta::hours(1),
            sut.backward_delta_chrono(d(2026, 6, 22, 9, 0, 0))
        );
        assert_eq!(
            TimeDelta::hours(71),
            sut.backward_delta_chrono(d(2026, 6, 22, 7, 0, 0))
        );
        assert_eq!(
            TimeDelta::hours(26),
            sut.backward_delta_chrono(d(2026, 6, 20, 10, 0, 0))
        );

        let overnight = Availability::anytime(WeekSlot::full());
        assert_eq!(
            TimeDelta::hours(9) + TimeDelta::minutes(30),
            overnight.backward_delta_chrono(d(2026, 6, 22, 9, 30, 0))
        );
    }

    #[test]
    fn test_next_window_start_at_or_after() {
        let sut = Availability::workdays(HourSlot { start: 8, stop: 12 });

        assert_eq!(
            d(2026, 6, 22, 9, 0, 0),
            sut.next_window_start(d(2026, 6, 22, 9, 0, 0))
        );
        assert_eq!(
            d(2026, 6, 22, 8, 0, 0),
            sut.next_window_start(d(2026, 6, 22, 7, 0, 0))
        );
        assert_eq!(
            d(2026, 6, 23, 8, 0, 0),
            sut.next_window_start(d(2026, 6, 22, 13, 0, 0))
        );
        assert_eq!(
            d(2026, 6, 22, 8, 0, 0),
            sut.next_window_start(d(2026, 6, 20, 10, 0, 0))
        );
    }

    #[test]
    fn test_window_end_after() {
        let sut = Availability::workdays(HourSlot { start: 8, stop: 12 });
        assert_eq!(
            Some(d(2026, 6, 22, 13, 0, 0)),
            sut.window_end_after(d(2026, 6, 22, 9, 30, 0))
        );

        let sut = Availability::new(WeekSlot::full(), HourSlot { start: 8, stop: 8 });
        assert_eq!(
            Some(d(2026, 6, 22, 9, 0, 0)),
            sut.window_end_after(d(2026, 6, 22, 8, 30, 0))
        );

        let sut = Availability::anytime(WeekSlot::workdays());
        assert_eq!(
            Some(d(2026, 6, 27, 0, 0, 0)),
            sut.window_end_after(d(2026, 6, 22, 9, 30, 0))
        );
    }

    #[test]
    fn test_can_fit() {
        let sut = Availability::workdays(HourSlot { start: 8, stop: 12 });
        assert!(sut.can_fit(d(2026, 6, 22, 11, 0, 0), Duration::hours(2)));
        assert!(!sut.can_fit(d(2026, 6, 22, 12, 30, 0), Duration::hours(1)));
        assert!(!sut.can_fit(d(2026, 6, 20, 10, 0, 0), Duration::hours(1)));
    }

    #[test]
    fn test_display() {
        assert_eq!(
            "08:00-12:00",
            Availability::new(WeekSlot::full(), HourSlot { start: 8, stop: 12 }).to_string()
        );
        assert_eq!(
            "Mon-Fri",
            Availability::anytime(WeekSlot::workdays()).to_string()
        );
        assert_eq!(
            "Mon-Fri 08:00-12:00",
            Availability::workdays(HourSlot { start: 8, stop: 12 }).to_string()
        );
    }

    #[test]
    fn test_serde_roundtrip() {
        let sut = Availability::workdays(HourSlot { start: 8, stop: 12 });
        let json = serde_json::to_string(&sut).unwrap();
        let back: Availability = serde_json::from_str(&json).unwrap();
        assert_eq!(sut, back);
    }
}
