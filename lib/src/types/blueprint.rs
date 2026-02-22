use crate::types::Duration;
use crate::types::HourSlot;
use crate::types::Priority;
use crate::types::Recurrence;
use crate::types::Slot;

/// A template for creating recurring tasks or events.
///
/// Blueprints define the core properties of a recurring item:
/// - How long it takes (`estimated_duration`)
/// - When it's preferred to be scheduled (`preferred_slot`)
/// - How often it repeats (`recurrence`)
/// - Its urgency level (`priority`)
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Blueprint {
    id: String,
    description: String,
    estimated_duration: Duration,
    priority: Priority,
    recurrence: Recurrence,
    preferred_slot: Slot,
}

impl Blueprint {
    pub fn new<S: Into<String>>(
        id: S,
        description: S,
        estimated_duration: Duration,
        priority: Priority,
        recurrence: Recurrence,
        preferred_slot: Slot,
    ) -> Self {
        Self {
            id: id.into(),
            description: description.into(),
            estimated_duration,
            priority,
            recurrence,
            preferred_slot,
        }
    }

    pub fn id(&self) -> &str {
        &self.id
    }

    pub fn description(&self) -> &str {
        &self.description
    }

    pub fn estimated_duration(&self) -> Duration {
        self.estimated_duration
    }

    pub fn priority(&self) -> Priority {
        self.priority
    }

    pub fn recurrence(&self) -> Recurrence {
        self.recurrence
    }

    pub fn preferred_slot(&self) -> Slot {
        self.preferred_slot
    }
}

impl std::fmt::Display for Blueprint {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!(
            "{} {} {} {} {}",
            self.id(),
            self.priority(),
            self.recurrence(),
            self.estimated_duration(),
            self.preferred_slot()
        ))
    }
}

#[cfg(test)]
mod test {

    use pretty_assertions::assert_eq;

    use super::*;
    use crate::types::TimeUnit;
    use crate::types::WeekSlot;

    #[test]
    fn test_display() {
        let sut = Blueprint::new(
            "1",
            "Clean VAC filters",
            Duration::hours(1),
            Priority::Idle,
            Recurrence::Period {
                spacing: Duration::of(1, TimeUnit::Year),
            },
            Slot::Hour(HourSlot::Range { start: 10, end: 13 }),
        );

        assert_eq!("1 IDLE ^1y 1h 10:00-13:00", sut.to_string());

        let sut = Blueprint::new(
            "1",
            "Clean VAC filters",
            Duration::hours(1),
            Priority::Crit,
            Recurrence::Period {
                spacing: Duration::of(3, TimeUnit::Month),
            },
            Slot::Week(WeekSlot::workdays()),
        );
        assert_eq!("1 CRIT ^3mo 1h Mon-Fri", sut.to_string());
    }
}
