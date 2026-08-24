use chrono::DateTime;
use chrono::Datelike;
use chrono::Days;
use chrono::Local;
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
    pub const fn new(days: WeekSlot, hours: HourSlot) -> Self {
        Self { days, hours }
    }

    const fn start_hour(&self) -> u32 {
        match self.hours {
            HourSlot::Fixed { hour } | HourSlot::Range { start: hour, .. } => hour,
        }
    }

    pub const fn workdays(hours: HourSlot) -> Self {
        Self::new(WeekSlot::workdays(), hours)
    }

    pub const fn anytime(days: WeekSlot) -> Self {
        Self::new(days, HourSlot::Range { start: 0, stop: 23 })
    }

    pub const fn full_week_all_day() -> Self {
        Self::new(WeekSlot::full(), HourSlot::Range { start: 0, stop: 23 })
    }

    pub const fn days(&self) -> WeekSlot {
        self.days
    }

    pub const fn hours(&self) -> HourSlot {
        self.hours
    }

    pub fn contains<T: TimeZone>(&self, ts: DateTime<T>) -> bool {
        self.days.matches_chrono(ts.clone()) && self.hours.matches_chrono(ts)
    }

    pub fn backward_delta_chrono(&self, ts: DateTime<Local>) -> TimeDelta {
        ts - self.most_recent_window_start(ts)
    }

    /// Returns the next timestamp at or after `ts` that lies inside this
    /// availability window.
    pub fn next_window_start_at_or_after(&self, ts: DateTime<Local>) -> DateTime<Local> {
        if self.contains(ts) {
            return ts;
        }

        let start_hour = self.start_hour();

        (0..=7)
            .filter_map(|days_fwd| {
                let date = ts.date_naive() + Days::new(days_fwd);
                let day = date.weekday().into();
                if !self.days.matches(day) {
                    return None;
                }

                let candidate = Local
                    .with_ymd_and_hms(date.year(), date.month(), date.day(), start_hour, 0, 0)
                    .unwrap();

                (candidate >= ts).then_some(candidate)
            })
            .min()
            .expect("availability must have a next matching window")
    }

    /// Returns the exclusive end of the current window containing `ts`.
    pub fn window_end_after(&self, ts: DateTime<Local>) -> Option<DateTime<Local>> {
        if !self.contains(ts) {
            return None;
        }

        let mut boundary = self.next_hour_boundary(ts);
        for _ in 0..=(24 * 8) {
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

    fn most_recent_window_start(&self, ts: DateTime<Local>) -> DateTime<Local> {
        let start_hour = self.start_hour();

        (0..=7)
            .filter_map(|days_back| {
                let date = ts.date_naive() - Days::new(days_back);
                let day = date.weekday().into();
                if !self.days.matches(day) {
                    return None;
                }

                let candidate = Local
                    .with_ymd_and_hms(date.year(), date.month(), date.day(), start_hour, 0, 0)
                    .unwrap();

                (candidate <= ts).then_some(candidate)
            })
            .max()
            .expect("availability must have a previous matching window")
    }

    fn next_hour_boundary(&self, ts: DateTime<Local>) -> DateTime<Local> {
        ts.with_minute(0)
            .and_then(|ts| ts.with_second(0))
            .and_then(|ts| ts.with_nanosecond(0))
            .unwrap()
            + TimeDelta::hours(1)
    }
}

impl std::fmt::Display for Availability {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let full_week = self.days == WeekSlot::full();
        let all_day = self.hours == HourSlot::Range { start: 0, stop: 23 };

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
        let sut = Availability::workdays(HourSlot::Range { start: 8, stop: 17 });

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
                HourSlot::Range { start: 0, stop: 23 }
            ),
            sut
        );

        assert_eq!(
            Availability::new(WeekSlot::full(), HourSlot::Range { start: 0, stop: 23 }),
            Availability::full_week_all_day()
        );
    }

    #[test]
    fn test_backward_delta_chrono() {
        let sut = Availability::workdays(HourSlot::Range { start: 8, stop: 12 });

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
        let sut = Availability::workdays(HourSlot::Range { start: 8, stop: 12 });

        assert_eq!(
            d(2026, 6, 22, 9, 0, 0),
            sut.next_window_start_at_or_after(d(2026, 6, 22, 9, 0, 0))
        );
        assert_eq!(
            d(2026, 6, 22, 8, 0, 0),
            sut.next_window_start_at_or_after(d(2026, 6, 22, 7, 0, 0))
        );
        assert_eq!(
            d(2026, 6, 23, 8, 0, 0),
            sut.next_window_start_at_or_after(d(2026, 6, 22, 13, 0, 0))
        );
        assert_eq!(
            d(2026, 6, 22, 8, 0, 0),
            sut.next_window_start_at_or_after(d(2026, 6, 20, 10, 0, 0))
        );
    }

    #[test]
    fn test_window_end_after() {
        let sut = Availability::workdays(HourSlot::Range { start: 8, stop: 12 });
        assert_eq!(
            Some(d(2026, 6, 22, 13, 0, 0)),
            sut.window_end_after(d(2026, 6, 22, 9, 30, 0))
        );

        let sut = Availability::new(WeekSlot::full(), HourSlot::Fixed { hour: 8 });
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
        let sut = Availability::workdays(HourSlot::Range { start: 8, stop: 12 });
        assert!(sut.can_fit(d(2026, 6, 22, 11, 0, 0), Duration::hours(2)));
        assert!(!sut.can_fit(d(2026, 6, 22, 12, 30, 0), Duration::hours(1)));
        assert!(!sut.can_fit(d(2026, 6, 20, 10, 0, 0), Duration::hours(1)));
    }

    #[test]
    fn test_display() {
        assert_eq!(
            "08:00-12:00",
            Availability::new(WeekSlot::full(), HourSlot::Range { start: 8, stop: 12 }).to_string()
        );
        assert_eq!(
            "Mon-Fri",
            Availability::anytime(WeekSlot::workdays()).to_string()
        );
        assert_eq!(
            "Mon-Fri 08:00-12:00",
            Availability::workdays(HourSlot::Range { start: 8, stop: 12 }).to_string()
        );
    }

    #[test]
    fn test_serde_roundtrip() {
        let sut = Availability::workdays(HourSlot::Range { start: 8, stop: 12 });
        let json = serde_json::to_string(&sut).unwrap();
        let back: Availability = serde_json::from_str(&json).unwrap();
        assert_eq!(sut, back);
    }
}
