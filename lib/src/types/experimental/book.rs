use std::cmp;

use chrono::DateTime;
use chrono::Local;
use chrono::TimeDelta;

use crate::sequencer::Sequencer;
use crate::types::Blueprint;

/// Models a collection of blueprints.
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Book {
    blueprints: Vec<Blueprint>,
}

impl Book {
    pub fn new(mut blueprints: Vec<Blueprint>) -> Self {
        blueprints.sort_by_key(|b| cmp::Reverse(b.priority()));
        Self { blueprints }
    }

    pub fn blueprints(&self) -> &[Blueprint] {
        &self.blueprints
    }

    pub fn spawn_sequencers(&self) -> Vec<(&Blueprint, Sequencer)> {
        self.blueprints
            .iter()
            .map(|b| (b, Sequencer::from_blueprint(b)))
            .collect()
    }

    pub fn min_fwd_delta_chrono(&self, ts: DateTime<Local>) -> Option<TimeDelta> {
        self.blueprints
            .iter()
            .map(|blueprint| blueprint.preferred_slot().fwd_delta_chrono(ts))
            .filter(|delta| !delta.is_zero())
            .min()
    }
}

impl std::fmt::Display for Book {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for bp in &self.blueprints {
            bp.fmt(f)?;
            f.write_str("\n")?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use chrono::Local;
    use chrono::TimeDelta;
    use chrono::TimeZone;

    use crate::types::Blueprint;
    use crate::types::Duration;
    use crate::types::HourSlot;
    use crate::types::Priority;
    use crate::types::Recurrence;
    use crate::types::Slot;
    use crate::types::TimeUnit;
    use crate::types::experimental::book::Book;

    #[test]
    fn test_min_fwd_delta_chrono() {
        let eight_am = Slot::Hour(HourSlot::Fixed { hour: 8 });
        let morning = Slot::Hour(HourSlot::Range { start: 8, stop: 12 });
        let daily = Recurrence::Period {
            spacing: Duration::of(1, TimeUnit::Day),
        };

        let one_hour = Duration::of(1, TimeUnit::Hour);

        let sut = Book::new(vec![
            Blueprint::new(
                "1".to_string(),
                "Task A".to_string(),
                one_hour,
                Priority::Crit,
                daily,
                eight_am,
            ),
            Blueprint::new(
                "2".to_string(),
                "Task B".to_string(),
                one_hour,
                Priority::Norm,
                daily,
                morning,
            ),
        ]);

        let ts = Local.with_ymd_and_hms(2025, 10, 23, 14, 0, 0).unwrap();
        assert_eq!(Some(TimeDelta::hours(18)), sut.min_fwd_delta_chrono(ts));
    }
}
