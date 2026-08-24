use chrono::DateTime;
use chrono::Local;

use crate::sequencer::Sequencer;
use crate::types::Blueprint;
use crate::types::experimental::book::Book;
use crate::types::experimental::journal::Journal;
use crate::types::experimental::plan::Plan;
use crate::types::experimental::plan_entry::PlanEntry;

pub struct Scheduler {
    sequencers: Vec<(Blueprint, Sequencer)>,
}

impl Scheduler {
    pub fn new(book: Book, journal: Journal) -> Self {
        Self {
            sequencers: book.spawn_sequencers(&journal),
        }
    }

    pub fn schedule(mut self, mut from: DateTime<Local>, to: DateTime<Local>) -> Plan {
        let mut entries: Vec<PlanEntry> = Vec::new();
        while from < to {
            let Some(entry) = self.sequence_next_entry_at_or_after(from) else {
                break;
            };
            if entry.planned_for() > to {
                break;
            }

            from = entry.planned_for() + entry.duration().timedelta();
            entries.push(entry);
        }

        Plan::new(entries)
    }

    pub fn sequence_next_entry_at_or_after(&mut self, ts: DateTime<Local>) -> Option<PlanEntry> {
        let (idx, planned_for) = self
            .sequencers
            .iter()
            .enumerate()
            .filter_map(|(idx, (blueprint, sequencer))| {
                self.next_fitting_candidate(blueprint, sequencer, ts)
                    .map(|candidate| (idx, candidate))
            })
            .min_by_key(|(idx, candidate)| (*candidate, *idx))?;

        let (blueprint, sequencer) = &mut self.sequencers[idx];
        let duration = blueprint.estimated_duration();

        debug_assert!(sequencer.accepts(planned_for));
        debug_assert!(blueprint.availability().can_fit(planned_for, duration));
        sequencer.commit(planned_for);

        Some(PlanEntry::new(
            blueprint.id().to_string(),
            duration,
            planned_for,
        ))
    }

    fn next_fitting_candidate(
        &self,
        blueprint: &Blueprint,
        sequencer: &Sequencer,
        ts: DateTime<Local>,
    ) -> Option<DateTime<Local>> {
        let availability = blueprint.availability();
        let duration = blueprint.estimated_duration();
        let mut candidate = sequencer.next_candidate_at_or_after(ts)?;
        let search_limit = candidate + chrono::TimeDelta::days(8);

        while candidate < search_limit {
            if availability.can_fit(candidate, duration) {
                return Some(candidate);
            }

            let window_end = availability.window_end_after(candidate)?;
            candidate = sequencer.next_candidate_at_or_after(window_end)?;
        }

        None
    }
}
#[cfg(test)]
mod test {

    use chrono::TimeDelta;
    use pretty_assertions::assert_eq;

    use super::*;
    use crate::test::d;
    use crate::types::Availability;
    use crate::types::Blueprint;
    use crate::types::Duration;
    use crate::types::HourSlot;
    use crate::types::Priority;
    use crate::types::Recurrence;
    use crate::types::TimeUnit;
    use crate::types::WeekSlot;
    use crate::types::experimental::journal::Commit;

    #[test]
    fn test_schedule() {
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
                Availability::new(WeekSlot::full(), HourSlot::Fixed { hour: 8 }),
            ),
            Blueprint::new(
                "2".into(),
                "Task B".into(),
                one_hour,
                Priority::Norm,
                daily,
                Availability::new(WeekSlot::full(), HourSlot::Range { start: 8, stop: 12 }),
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
        assert_eq!(expected.trim(), plan.as_str().trim());
    }

    #[test]
    fn test_schedule_hourly_task_one_day() {
        let bp_hourly = Blueprint::new(
            "id-1".into(),
            "Hourly Task".into(),
            Duration::hours(2) + Duration::minutes(30),
            Priority::Norm,
            Recurrence::Period {
                spacing: Duration::hours(1),
            },
            Availability::full_week_all_day(),
        );

        let book = Book::new(vec![bp_hourly]);
        let journal = Journal::new(vec![Commit::completed(
            "id-1".into(),
            d(2026, 6, 15, 1, 30, 0),
        )]);

        let from = d(2026, 6, 15, 0, 0, 0);
        let plan = Scheduler::new(book, journal).schedule(from, from + TimeDelta::days(1));

        let expected = "
id-1 2026-06-15T02:30:00+02:00
id-1 2026-06-15T05:00:00+02:00
id-1 2026-06-15T07:30:00+02:00
id-1 2026-06-15T10:00:00+02:00
id-1 2026-06-15T12:30:00+02:00
id-1 2026-06-15T15:00:00+02:00
id-1 2026-06-15T17:30:00+02:00
id-1 2026-06-15T20:00:00+02:00
id-1 2026-06-15T22:30:00+02:00";

        assert_eq!(expected.trim(), plan.as_str().trim());
    }

    #[test]
    fn test_schedule_availability_only_blueprint() {
        let book = Book::new(vec![Blueprint::new(
            "1".into(),
            "Workday Task".into(),
            Duration::hours(1),
            Priority::Crit,
            Recurrence::Period {
                spacing: Duration::days(1),
            },
            Availability::workdays(HourSlot::Range { start: 8, stop: 12 }),
        )]);

        let from = d(2026, 6, 20, 10, 0, 0);
        let plan =
            Scheduler::new(book, Journal::new(vec![])).schedule(from, from + TimeDelta::days(3));

        let expected = "
1 2026-06-22T08:00:00+02:00
1 2026-06-23T08:00:00+02:00";

        assert_eq!(expected.trim(), plan.as_str().trim());
    }

    #[test]
    fn test_schedule_skips_candidate_that_does_not_fit_window() {
        let book = Book::new(vec![
            Blueprint::new(
                "1".into(),
                "First Morning Task".into(),
                Duration::minutes(90),
                Priority::Crit,
                Recurrence::Once,
                Availability::workdays(HourSlot::Range { start: 8, stop: 10 }),
            ),
            Blueprint::new(
                "2".into(),
                "Second Morning Task".into(),
                Duration::hours(2),
                Priority::Norm,
                Recurrence::Once,
                Availability::workdays(HourSlot::Range { start: 8, stop: 10 }),
            ),
        ]);

        let from = d(2026, 6, 22, 8, 0, 0);
        let plan =
            Scheduler::new(book, Journal::new(vec![])).schedule(from, from + TimeDelta::days(2));

        let expected = "
1 2026-06-22T08:00:00+02:00
2 2026-06-23T08:00:00+02:00";

        assert_eq!(expected.trim(), plan.as_str().trim());
    }

    #[test]
    fn test_schedule_drops_task_that_can_never_fit_any_window() {
        let book = Book::new(vec![Blueprint::new(
            "1".into(),
            "Impossible Morning Task".into(),
            Duration::hours(4),
            Priority::Crit,
            Recurrence::Once,
            Availability::workdays(HourSlot::Range { start: 8, stop: 10 }),
        )]);

        let from = d(2026, 6, 22, 8, 0, 0);
        let plan =
            Scheduler::new(book, Journal::new(vec![])).schedule(from, from + TimeDelta::days(2));

        assert_eq!("", plan.as_str().trim());
    }
}
