use std::collections::HashMap;
use std::collections::VecDeque;

use chrono::Days;
use chrono::Local;
use chrono::TimeDelta;
use chrono::TimeZone;
use chrono::Timelike;

use crate::types::BlueprintBook;
use crate::types::HourSlot;
use crate::types::Journal;
use crate::types::Plan;
use crate::types::PlanEntry;
use crate::types::Recurrence;
use crate::types::Slot;

pub fn schedule(book: BlueprintBook, _journal: Journal) -> Plan {
    let mut blueprint_plans: HashMap<String, VecDeque<PlanEntry>> = HashMap::new();

    // let now = Local::now();
    let now = Local.with_ymd_and_hms(2026, 10, 23, 0, 0, 0).unwrap();
    let end = now.checked_add_days(Days::new(7)).unwrap();

    for blueprint in book.entries() {
        // todo: skip entries in journal

        let mut entries: VecDeque<PlanEntry> = VecDeque::new();

        let mut ts = now;
        while ts < end {
            match blueprint.preferred_slot() {
                Slot::Hour(hour_slot) => match hour_slot {
                    HourSlot::Fixed { hour } | HourSlot::Range { start: hour, .. } => {
                        if ts.hour() != hour as u32 {
                            ts = ts
                                .checked_add_signed(TimeDelta::hours(
                                    hour as i64 - ts.hour() as i64,
                                ))
                                .unwrap();
                        }
                    }
                },
                Slot::Week(week_slot) => todo!(),
            }

            let entry = PlanEntry::new(blueprint.id(), ts);
            entries.push_back(entry);
            match blueprint.recurrence() {
                Recurrence::Once => break,
                Recurrence::Period { spacing } => {
                    ts += spacing.timedelta();
                }
                Recurrence::Times { count, spacing } => unimplemented!(),
            }
        }

        blueprint_plans.insert(blueprint.id().to_string(), entries);
    }

    let mut linearized: Vec<PlanEntry> = Vec::new();
    while !blueprint_plans.values().all(|v| v.is_empty()) {
        let mut closer: Option<&PlanEntry> = None;
        for blueprint in book.entries() {
            if let Some(candidate) = blueprint_plans[blueprint.id()].front() {
                match closer {
                    Some(best) => {
                        if best.planned_for() > candidate.planned_for() {
                            closer = Some(candidate);
                        }
                    }
                    None => closer = Some(candidate),
                }
            }
        }

        let winner = closer.unwrap().clone();
        blueprint_plans
            .get_mut(winner.blueprint_id())
            .unwrap()
            .pop_front();
        linearized.push(winner);
    }

    Plan::from(linearized)
}

#[cfg(test)]
mod test {

    use pretty_assertions::assert_eq;

    use super::*;
    use crate::types::Blueprint;
    use crate::types::Duration;
    use crate::types::HourSlot;
    use crate::types::Priority;
    use crate::types::Recurrence;
    use crate::types::Slot;
    use crate::types::TimeUnit;

    #[test]
    fn test_schedule() {
        let eight_am = Slot::Hour(HourSlot::Fixed { hour: 8 });
        let morning = Slot::Hour(HourSlot::Range { start: 8, end: 12 });
        let daily = Recurrence::Period {
            spacing: Duration::of(1, TimeUnit::Day),
        };

        let one_hour = Duration::of(1, TimeUnit::Hour);

        let book = BlueprintBook::from(vec![
            Blueprint::new("1", "Task A", one_hour, Priority::Crit, daily, eight_am),
            Blueprint::new("2", "Task B", one_hour, Priority::Norm, daily, morning),
        ]);

        let expected = "

1 CRIT ^1d 1h 08:00
2 NORM ^1d 1h 08:00-12:00
";
        assert_eq!(expected.trim(), book.to_string().trim());

        let journal = Journal::from(vec![]);

        let plan = schedule(book, journal);

        let expected = "
1 2026-10-23T08:00:00+02:00
2 2026-10-23T08:00:00+02:00
1 2026-10-24T08:00:00+02:00
2 2026-10-24T08:00:00+02:00
1 2026-10-25T08:00:00+01:00
2 2026-10-25T08:00:00+01:00
1 2026-10-26T08:00:00+01:00
2 2026-10-26T08:00:00+01:00
1 2026-10-27T08:00:00+01:00
2 2026-10-27T08:00:00+01:00
1 2026-10-28T08:00:00+01:00
2 2026-10-28T08:00:00+01:00
1 2026-10-29T08:00:00+01:00
2 2026-10-29T08:00:00+01:00";

        assert_eq!(expected.trim(), plan.as_str().trim());
    }
}
