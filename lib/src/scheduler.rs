use std::collections::HashSet;

use chrono::DateTime;
use chrono::Local;

use crate::sequencer::Sequencer;
use crate::types::Blueprint;
use crate::types::experimental::book::Book;
use crate::types::experimental::journal::Journal;
use crate::types::experimental::plan::Plan;
use crate::types::experimental::plan_entry::PlanEntry;

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct ScheduleReport {
    plan: Plan,
    warnings: Vec<ScheduleWarning>,
}

impl ScheduleReport {
    pub fn new(plan: Plan, warnings: Vec<ScheduleWarning>) -> Self {
        Self { plan, warnings }
    }

    pub fn plan(&self) -> &Plan {
        &self.plan
    }

    pub fn warnings(&self) -> &[ScheduleWarning] {
        &self.warnings
    }

    pub fn items(&self) -> Vec<ScheduleItem> {
        let mut items: Vec<_> = self
            .warnings
            .iter()
            .cloned()
            .map(ScheduleItem::Warning)
            .chain(self.plan.entries().iter().cloned().map(ScheduleItem::Entry))
            .collect();
        items.sort_by_key(ScheduleItem::at);
        items
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum ScheduleItem {
    Warning(ScheduleWarning),
    Entry(PlanEntry),
}

impl ScheduleItem {
    pub fn at(&self) -> DateTime<Local> {
        match self {
            Self::Warning(warning) => warning.at(),
            Self::Entry(entry) => entry.planned_for(),
        }
    }
}

impl std::fmt::Display for ScheduleItem {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Warning(warning) => warning.fmt(f),
            Self::Entry(entry) => write!(
                f,
                "{} {}",
                entry.blueprint_id(),
                entry.planned_for().to_rfc3339()
            ),
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct ScheduleWarning {
    blueprint: Blueprint,
    at: DateTime<Local>,
}

impl ScheduleWarning {
    fn unplannable(blueprint: &Blueprint, at: DateTime<Local>) -> Self {
        Self {
            blueprint: blueprint.clone(),
            at,
        }
    }

    pub fn blueprint(&self) -> &Blueprint {
        &self.blueprint
    }

    pub fn at(&self) -> DateTime<Local> {
        self.at
    }
}

impl std::fmt::Display for ScheduleWarning {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "(!) {}", self.blueprint)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum CandidateSearch {
    Fit(DateTime<Local>),
    Impossible { first_candidate: DateTime<Local> },
    Exhausted,
}

pub struct Scheduler {
    sequencers: Vec<(Blueprint, Sequencer)>,
}

impl Scheduler {
    pub fn new(book: Book, journal: Journal) -> Self {
        Self {
            sequencers: book.spawn_sequencers(&journal),
        }
    }

    pub fn schedule(self, from: DateTime<Local>, to: DateTime<Local>) -> Plan {
        self.schedule_with_warnings(from, to).plan
    }

    pub fn schedule_with_warnings(
        mut self,
        mut from: DateTime<Local>,
        to: DateTime<Local>,
    ) -> ScheduleReport {
        let mut entries: Vec<PlanEntry> = Vec::new();
        let mut warnings = Vec::new();
        let mut warned_unplannable_blueprints = HashSet::new();

        while from < to {
            let Some(entry) = self.sequence_next_entry_at_or_after(
                from,
                to,
                &mut warnings,
                &mut warned_unplannable_blueprints,
            ) else {
                break;
            };

            from = entry.planned_for() + entry.duration().timedelta();
            entries.push(entry);
        }

        ScheduleReport::new(Plan::new(entries), warnings)
    }

    fn sequence_next_entry_at_or_after(
        &mut self,
        ts: DateTime<Local>,
        to: DateTime<Local>,
        warnings: &mut Vec<ScheduleWarning>,
        warned_unplannable_blueprints: &mut HashSet<String>,
    ) -> Option<PlanEntry> {
        let candidates: Vec<_> = self
            .sequencers
            .iter()
            .enumerate()
            .map(|(idx, (blueprint, sequencer))| {
                (idx, self.next_fitting_candidate(blueprint, sequencer, ts))
            })
            .collect();

        for (idx, candidate) in &candidates {
            let blueprint = &self.sequencers[*idx].0;
            if let CandidateSearch::Impossible { first_candidate } = candidate
                && *first_candidate <= to
                && warned_unplannable_blueprints.insert(blueprint.id().to_string())
            {
                warnings.push(ScheduleWarning::unplannable(blueprint, *first_candidate));
            }
        }

        let (idx, planned_for) = candidates
            .iter()
            .filter_map(|(idx, candidate)| match candidate {
                CandidateSearch::Fit(candidate) if *candidate <= to => Some((*idx, *candidate)),
                CandidateSearch::Fit(_)
                | CandidateSearch::Impossible { .. }
                | CandidateSearch::Exhausted => None,
            })
            .min_by_key(|(idx, candidate)| (*candidate, *idx))?;

        let next_from = planned_for + self.sequencers[idx].0.estimated_duration().timedelta();
        for (other_idx, candidate) in &candidates {
            if *other_idx == idx {
                continue;
            }

            let blueprint = &self.sequencers[*other_idx].0;
            if let CandidateSearch::Fit(candidate) = candidate
                && *candidate <= to
                && *candidate < next_from
            {
                warnings.push(ScheduleWarning::unplannable(blueprint, *candidate));
            }
        }

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
    ) -> CandidateSearch {
        let availability = blueprint.availability();
        let duration = blueprint.estimated_duration();
        let Some(mut candidate) = sequencer.next_candidate_for(ts) else {
            return CandidateSearch::Exhausted;
        };
        let first_candidate = candidate;
        let search_limit = candidate + chrono::TimeDelta::days(8);

        while candidate < search_limit {
            if availability.can_fit(candidate, duration) {
                return CandidateSearch::Fit(candidate);
            }

            let Some(window_end) = availability.window_end_after(candidate) else {
                break;
            };
            let Some(next_candidate) = sequencer.next_candidate_for(window_end) else {
                return CandidateSearch::Exhausted;
            };
            candidate = next_candidate;
        }

        CandidateSearch::Impossible { first_candidate }
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

    #[test]
    fn test_schedule_warns_when_due_task_can_never_fit_any_window() {
        let book = Book::new(vec![Blueprint::new(
            "1".into(),
            "Impossible Morning Task".into(),
            Duration::hours(4),
            Priority::Crit,
            Recurrence::Once,
            Availability::workdays(HourSlot::Range { start: 8, stop: 10 }),
        )]);

        let from = d(2026, 6, 22, 8, 0, 0);
        let report = Scheduler::new(book, Journal::new(vec![]))
            .schedule_with_warnings(from, from + TimeDelta::days(2));

        assert_eq!("", report.plan().as_str().trim());
        assert_eq!(1, report.warnings().len());
        assert_eq!("1", report.warnings()[0].blueprint().id());
        assert_eq!(
            "(!) 1 CRIT ^1 4h Mon-Fri 08:00-10:00",
            report.warnings()[0].to_string()
        );
    }

    #[test]
    fn test_schedule_does_not_warn_for_impossible_task_outside_range() {
        let book = Book::new(vec![Blueprint::new(
            "1".into(),
            "Impossible Morning Task".into(),
            Duration::hours(4),
            Priority::Crit,
            Recurrence::Once,
            Availability::workdays(HourSlot::Range { start: 8, stop: 10 }),
        )]);

        let from = d(2026, 6, 22, 0, 0, 0);
        let report = Scheduler::new(book, Journal::new(vec![]))
            .schedule_with_warnings(from, d(2026, 6, 22, 7, 0, 0));

        assert_eq!("", report.plan().as_str().trim());
        assert!(report.warnings().is_empty());
    }

    #[test]
    fn test_schedule_warns_for_every_conflicted_occurrence() {
        let book = Book::new(vec![
            Blueprint::new(
                "2".into(),
                "Team standup".into(),
                Duration::minutes(30),
                Priority::High,
                Recurrence::Period {
                    spacing: Duration::days(1),
                },
                Availability::workdays(HourSlot::Range {
                    start: 10,
                    stop: 11,
                }),
            ),
            Blueprint::new(
                "1".into(),
                "Deep work".into(),
                Duration::hours(4),
                Priority::Norm,
                Recurrence::Period {
                    spacing: Duration::days(1),
                },
                Availability::new(WeekSlot::full(), HourSlot::Range { start: 8, stop: 12 }),
            ),
        ]);

        let from = d(2026, 8, 25, 0, 0, 0);
        let report = Scheduler::new(book, Journal::new(vec![]))
            .schedule_with_warnings(from, from + TimeDelta::days(7));

        assert_eq!(5, report.warnings().len());
        assert!(
            report
                .warnings()
                .iter()
                .all(|warning| warning.to_string() == "(!) 2 HIGH ^1d 30min Mon-Fri 10:00-11:00")
        );

        let rendered: Vec<_> = report.items().iter().map(ToString::to_string).collect();
        assert_eq!(
            vec![
                "1 2026-08-25T08:00:00+02:00",
                "(!) 2 HIGH ^1d 30min Mon-Fri 10:00-11:00",
                "1 2026-08-26T08:00:00+02:00",
                "(!) 2 HIGH ^1d 30min Mon-Fri 10:00-11:00",
            ],
            rendered[..4]
        );
    }
}
