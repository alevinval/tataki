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
                        // No delta available, so we're inside a slot, but the Journal commits do
                        // not allow the sequencer to schedule anything yet.
                        panic!("unexpected state");
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
    use pretty_assertions::assert_eq;

    use super::*;
    use crate::test::d;
    use crate::types::Blueprint;
    use crate::types::Duration;
    use crate::types::HourSlot;
    use crate::types::Priority;
    use crate::types::Recurrence;
    use crate::types::Slot;
    use crate::types::TimeUnit;
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
            Commit::completed("2".into(), d(2026, 10, 25, 18, 30, 0)),
        ]);

        let plan = Scheduler::new(book, journal).schedule(from, from + TimeDelta::days(7));
        let expected = "
1 2026-10-23T08:00:00+02:00
1 2026-10-24T08:00:00+02:00
1 2026-10-25T08:00:00+01:00
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
}
