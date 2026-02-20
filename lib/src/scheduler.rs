use chrono::DateTime;
use chrono::Local;

use crate::sequencer::Sequencer;
use crate::types::Blueprint;
use crate::types::experimental::book::Book;
use crate::types::experimental::journal::Journal;
use crate::types::experimental::plan::Plan;
use crate::types::experimental::plan_entry::PlanEntry;

pub struct Scheduler {
    book: Book,
    journal: Journal,
    sequencers: Vec<(Blueprint, Sequencer)>,
}

impl Scheduler {
    pub fn new(book: Book, journal: Journal) -> Self {
        let sequencers = book.spawn_sequencers(&journal);

        Self {
            book,
            journal,
            sequencers,
        }
    }

    pub fn schedule(mut self, mut from: DateTime<Local>, to: DateTime<Local>) -> Plan {
        let mut entries: Vec<PlanEntry> = Vec::new();
        while from < to {
            match self.sequence_next_entry(from) {
                Some(entry) => {
                    if entry.planned_for() > to {
                        break;
                    }

                    from += entry.duration().timedelta();
                    entries.push(entry);
                }
                None => {
                    if let Some(delta) = self.book.min_fwd_delta_chrono(from) {
                        from += delta;
                    } else {
                        panic!("...");
                    }
                }
            }
        }

        Plan::new(entries)
    }

    pub fn sequence_next_entry(&mut self, ts: DateTime<Local>) -> Option<PlanEntry> {
        self.sequencers
            .iter_mut()
            .find(|(_, sequencer)| sequencer.accepts(ts))
            .map(|(blueprint, sequencer)| {
                sequencer.commit(ts);
                PlanEntry::new(
                    blueprint.id().to_string(),
                    blueprint.estimated_duration(),
                    ts,
                )
            })
    }
}
#[cfg(test)]
mod test {

    use chrono::TimeDelta;
    use chrono::TimeZone;
    use pretty_assertions::assert_eq;

    use super::*;
    use crate::types::Blueprint;
    use crate::types::Duration;
    use crate::types::HourSlot;
    use crate::types::Priority;
    use crate::types::Recurrence;
    use crate::types::Slot;
    use crate::types::TimeUnit;
    use crate::types::WeekSlot;
    use crate::types::days::DayOfWeek;
    use crate::types::experimental::journal::Commit;

    #[test]
    fn test_schedule() {
        let eight_am = Slot::Hour(HourSlot::Fixed { hour: 8 });
        let morning = Slot::Hour(HourSlot::Range { start: 8, stop: 12 });
        let daily = Recurrence::Period {
            spacing: Duration::of(1, TimeUnit::Day),
        };

        let one_hour = Duration::of(1, TimeUnit::Hour);

        let book = Book::new(vec![
            Blueprint::new(
                "1".into(),
                "Task A".into(),
                one_hour,
                Priority::Crit,
                daily,
                eight_am,
            ),
            Blueprint::new(
                "2".into(),
                "Task B".into(),
                one_hour,
                Priority::Norm,
                daily,
                morning,
            ),
        ]);

        let expected = "

1 CRIT ^1d 1h 08:00
2 NORM ^1d 1h 08:00-12:00
";
        assert_eq!(expected.trim(), book.to_string().trim());

        let empty_journal = Journal::new(vec![]);
        let from = d(2026, 10, 23, 0, 0, 0);
        let plan =
            Scheduler::new(book.clone(), empty_journal).schedule(from, from + TimeDelta::days(7));

        let expected = "
1 2026-10-23T08:00:00+02:00
2 2026-10-23T09:00:00+02:00
1 2026-10-24T08:00:00+02:00
2 2026-10-24T09:00:00+02:00
1 2026-10-25T08:00:00+01:00
2 2026-10-25T09:00:00+01:00
1 2026-10-26T08:00:00+01:00
2 2026-10-26T09:00:00+01:00
1 2026-10-27T08:00:00+01:00
2 2026-10-27T09:00:00+01:00
1 2026-10-28T08:00:00+01:00
2 2026-10-28T09:00:00+01:00
1 2026-10-29T08:00:00+01:00
2 2026-10-29T09:00:00+01:00";

        assert_eq!(expected.trim(), plan.as_str().trim());

        let journal = Journal::new(vec![
            Commit::completed("1".into(), d(2026, 10, 22, 9, 0, 0)),
            Commit::completed("2".into(), d(2026, 10, 22, 9, 30, 0)),
        ]);

        let plan = Scheduler::new(book, journal).schedule(from, from + TimeDelta::days(7));
        let expected = "
1 2026-10-23T08:00:00+02:00
2 2026-10-23T09:00:00+02:00
1 2026-10-24T08:00:00+02:00
2 2026-10-24T09:00:00+02:00
1 2026-10-25T08:00:00+01:00
2 2026-10-25T09:00:00+01:00
1 2026-10-26T08:00:00+01:00
2 2026-10-26T09:00:00+01:00
1 2026-10-27T08:00:00+01:00
2 2026-10-27T09:00:00+01:00
1 2026-10-28T08:00:00+01:00
2 2026-10-28T09:00:00+01:00
1 2026-10-29T08:00:00+01:00
2 2026-10-29T09:00:00+01:00";

        assert_eq!(expected.trim(), plan.as_str().trim());
    }

    #[test]
    fn test_schedule_complex_weekly_plan() {
        // Weekly day-of-week constraints are expressed via HourSlot + daily/weekly
        // spacing rather than WeekSlot, because WeekSlot has no hour anchor
        // (backward_delta_chrono = 0 → infinite commits).

        let bp_standup = Blueprint::new(
            "standup".into(),
            "Daily Standup".into(),
            Duration::minutes(30),
            Priority::Crit,
            Recurrence::Period {
                spacing: Duration::days(1),
            },
            Slot::Hour(HourSlot::Fixed { hour: 9 }),
        );

        let bp_review = Blueprint::new(
            "review".into(),
            "Code Review".into(),
            Duration::hours(1),
            Priority::Norm,
            Recurrence::Period {
                spacing: Duration::days(1),
            },
            Slot::Hour(HourSlot::Range {
                start: 14,
                stop: 16,
            }),
        );

        let bp_report = Blueprint::new(
            "report".into(),
            "Weekly Report".into(),
            Duration::hours(2),
            Priority::Crit,
            Recurrence::Times {
                count: 4,
                spacing: Duration::of(7, TimeUnit::Day),
            },
            Slot::Week(WeekSlot::Fixed {
                day: DayOfWeek::Mon,
            }),
        );

        let bp_gym = Blueprint::new(
            "gym".into(),
            "Gym Session".into(),
            Duration::hours(1),
            Priority::Norm,
            Recurrence::Period {
                spacing: Duration::days(1),
            },
            Slot::Hour(HourSlot::Fixed { hour: 6 }),
        );

        let bp_clean = Blueprint::new(
            "clean".into(),
            "Clean Apartment".into(),
            Duration::hours(2),
            Priority::Idle,
            Recurrence::Times {
                count: 3,
                spacing: Duration::of(7, TimeUnit::Day),
            },
            Slot::Hour(HourSlot::Range {
                start: 10,
                stop: 12,
            }),
        );

        let bp_invoice = Blueprint::new(
            "invoice".into(),
            "Pay Invoices".into(),
            Duration::minutes(30),
            Priority::Crit,
            Recurrence::Once,
            Slot::Hour(HourSlot::Fixed { hour: 15 }),
        );

        let bp_sync = Blueprint::new(
            "sync".into(),
            "Team Sync".into(),
            Duration::hours(1),
            Priority::Norm,
            Recurrence::Period {
                spacing: Duration::of(7, TimeUnit::Day),
            },
            Slot::Hour(HourSlot::Fixed { hour: 11 }),
        );

        let bp_social = Blueprint::new(
            "social".into(),
            "Weekend Social".into(),
            Duration::hours(1),
            Priority::Idle,
            Recurrence::Period {
                spacing: Duration::of(7, TimeUnit::Day),
            },
            Slot::Hour(HourSlot::Range {
                start: 20,
                stop: 23,
            }),
        );

        let bp_meds = Blueprint::new(
            "meds".into(),
            "Medication".into(),
            Duration::minutes(5),
            Priority::Crit,
            Recurrence::Period {
                spacing: Duration::hours(8),
            },
            Slot::Hour(HourSlot::Fixed { hour: 8 }),
        );

        let bp_tax = Blueprint::new(
            "tax".into(),
            "Tax Preparation".into(),
            Duration::hours(3),
            Priority::Norm,
            Recurrence::Once,
            Slot::Hour(HourSlot::Range {
                start: 10,
                stop: 14,
            }),
        );

        let book = Book::new(vec![
            bp_standup, bp_review, bp_report, bp_gym, bp_clean, bp_invoice, bp_sync, bp_social,
            bp_meds, bp_tax,
        ]);

        // Prior completions shift the phase of gym and clean
        let journal = Journal::new(vec![
            Commit::completed("gym".into(), d(2026, 10, 20, 7, 0, 0)),
            Commit::completed("clean".into(), d(2026, 10, 11, 10, 0, 0)),
        ]);

        // Oct 24 (Sat) → Nov 14 (Sat), crossing the DST transition on Oct 25
        let from = d(2026, 10, 24, 0, 0, 0);
        let plan = Scheduler::new(book, journal).schedule(from, from + TimeDelta::days(21));

        // Cascading: each entry advances `from` by its duration, so later
        // tasks drift forward as earlier ones push the clock.
        let expected = r#"gym 2026-10-24T06:00:00+02:00
meds 2026-10-24T08:00:00+02:00
standup 2026-10-24T09:05:00+02:00
tax 2026-10-24T10:35:00+02:00
review 2026-10-24T14:35:00+02:00
invoice 2026-10-24T15:35:00+02:00
social 2026-10-24T20:05:00+02:00
gym 2026-10-25T06:05:00+01:00
meds 2026-10-25T08:05:00+01:00
standup 2026-10-25T09:10:00+01:00
clean 2026-10-25T10:40:00+01:00
review 2026-10-25T14:40:00+01:00
report 2026-10-26T06:40:00+01:00
meds 2026-10-26T08:40:00+01:00
standup 2026-10-26T09:45:00+01:00
sync 2026-10-26T11:15:00+01:00
review 2026-10-26T15:15:00+01:00
gym 2026-10-27T06:15:00+01:00
meds 2026-10-27T08:15:00+01:00
review 2026-10-27T14:20:00+01:00
gym 2026-10-28T06:20:00+01:00
meds 2026-10-28T08:20:00+01:00
standup 2026-10-28T09:25:00+01:00
review 2026-10-28T14:55:00+01:00
gym 2026-10-29T06:55:00+01:00
meds 2026-10-29T08:55:00+01:00
review 2026-10-29T15:00:00+01:00
meds 2026-10-30T08:00:00+01:00
standup 2026-10-30T09:05:00+01:00
review 2026-10-30T14:35:00+01:00
gym 2026-10-31T06:35:00+01:00
meds 2026-10-31T08:35:00+01:00
standup 2026-10-31T09:40:00+01:00
review 2026-10-31T15:10:00+01:00
social 2026-10-31T20:10:00+01:00
meds 2026-11-01T08:10:00+01:00
clean 2026-11-01T11:15:00+01:00
review 2026-11-01T14:15:00+01:00
gym 2026-11-02T06:15:00+01:00
report 2026-11-02T07:15:00+01:00
standup 2026-11-02T09:15:00+01:00
sync 2026-11-02T11:45:00+01:00
review 2026-11-02T14:45:00+01:00
gym 2026-11-03T06:45:00+01:00
meds 2026-11-03T08:45:00+01:00
standup 2026-11-03T09:50:00+01:00
review 2026-11-03T15:20:00+01:00
meds 2026-11-04T08:20:00+01:00
review 2026-11-04T14:25:00+01:00
gym 2026-11-05T06:25:00+01:00
meds 2026-11-05T08:25:00+01:00
standup 2026-11-05T09:30:00+01:00
review 2026-11-05T15:00:00+01:00
meds 2026-11-06T08:00:00+01:00
review 2026-11-06T14:05:00+01:00
gym 2026-11-07T06:05:00+01:00
meds 2026-11-07T08:05:00+01:00
standup 2026-11-07T09:10:00+01:00
review 2026-11-07T14:40:00+01:00
social 2026-11-07T20:40:00+01:00
gym 2026-11-08T06:40:00+01:00
meds 2026-11-08T08:40:00+01:00
standup 2026-11-08T09:45:00+01:00
clean 2026-11-08T10:15:00+01:00
review 2026-11-08T15:15:00+01:00
report 2026-11-09T08:15:00+01:00
review 2026-11-09T14:15:00+01:00
gym 2026-11-10T06:15:00+01:00
meds 2026-11-10T08:15:00+01:00
standup 2026-11-10T09:20:00+01:00
sync 2026-11-10T11:50:00+01:00
review 2026-11-10T14:50:00+01:00
gym 2026-11-11T06:50:00+01:00
meds 2026-11-11T08:50:00+01:00
standup 2026-11-11T09:55:00+01:00
review 2026-11-11T15:25:00+01:00
meds 2026-11-12T08:25:00+01:00
review 2026-11-12T14:30:00+01:00
gym 2026-11-13T06:30:00+01:00
meds 2026-11-13T08:30:00+01:00
standup 2026-11-13T09:35:00+01:00
review 2026-11-13T15:05:00+01:00"#;

        assert_eq!(expected.trim(), plan.as_str().trim());
    }

    fn d(year: i32, month: u32, day: u32, hour: u32, minute: u32, sec: u32) -> DateTime<Local> {
        Local
            .with_ymd_and_hms(year, month, day, hour, minute, sec)
            .unwrap()
    }
}
