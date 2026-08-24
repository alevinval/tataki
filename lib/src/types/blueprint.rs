use serde::Deserialize;
use serde::Serialize;

use crate::types::Availability;
use crate::types::Duration;
use crate::types::Priority;
use crate::types::Recurrence;

/// A template for creating recurring tasks or events.
///
/// Blueprints define the core properties of a recurring item:
/// - How long it takes (`estimated_duration`)
/// - When it may be scheduled (`availability`)
/// - How often it repeats (`recurrence`)
/// - Its urgency level (`priority`)
#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize)]
pub struct Blueprint {
    id: String,
    description: String,
    estimated_duration: Duration,
    priority: Priority,
    recurrence: Recurrence,
    availability: Availability,
}

impl Blueprint {
    pub fn new(
        id: String,
        description: String,
        estimated_duration: Duration,
        priority: Priority,
        recurrence: Recurrence,
        availability: Availability,
    ) -> Self {
        Self {
            id,
            description,
            estimated_duration,
            priority,
            recurrence,
            availability,
        }
    }

    pub fn id(&self) -> &str {
        &self.id
    }

    pub fn description(&self) -> &str {
        &self.description
    }

    pub const fn estimated_duration(&self) -> Duration {
        self.estimated_duration
    }

    pub const fn priority(&self) -> Priority {
        self.priority
    }

    pub const fn recurrence(&self) -> Recurrence {
        self.recurrence
    }

    pub const fn availability(&self) -> Availability {
        self.availability
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
            self.availability()
        ))
    }
}

#[cfg(test)]
mod test {

    use pretty_assertions::assert_eq;

    use super::*;
    use crate::types::HourSlot;
    use crate::types::TimeUnit;
    use crate::types::WeekSlot;

    fn get_example_blueprint() -> Blueprint {
        Blueprint::new(
            "1".to_string(),
            "Clean VAC filters".to_string(),
            Duration::hours(1),
            Priority::Idle,
            Recurrence::Period {
                spacing: Duration::of(1, TimeUnit::Year),
            },
            Availability::new(
                WeekSlot::full(),
                HourSlot::Range {
                    start: 10,
                    stop: 13,
                },
            ),
        )
    }

    #[test]
    fn test_display() {
        let sut = get_example_blueprint();
        assert_eq!("1 IDLE ^1y 1h 10:00-13:00", sut.to_string());

        let sut = Blueprint::new(
            "1".to_string(),
            "Clean VAC filters".to_string(),
            Duration::hours(1),
            Priority::Crit,
            Recurrence::Period {
                spacing: Duration::of(3, TimeUnit::Month),
            },
            Availability::anytime(WeekSlot::workdays()),
        );
        assert_eq!("1 CRIT ^3mo 1h Mon-Fri", sut.to_string());

        let sut = Blueprint::new(
            "1".to_string(),
            "Clean VAC filters".to_string(),
            Duration::hours(1),
            Priority::Crit,
            Recurrence::Period {
                spacing: Duration::of(3, TimeUnit::Month),
            },
            Availability::workdays(HourSlot::Range { start: 8, stop: 12 }),
        );
        assert_eq!("1 CRIT ^3mo 1h Mon-Fri 08:00-12:00", sut.to_string());
    }

    #[test]
    fn test_serde_roundtrip() {
        let sut = get_example_blueprint();
        let json = serde_json::to_string(&sut).unwrap();
        let back: Blueprint = serde_json::from_str(&json).unwrap();
        assert_eq!(sut, back);

        let sut = Blueprint::new(
            "1".to_string(),
            "Clean VAC filters".to_string(),
            Duration::hours(1),
            Priority::Crit,
            Recurrence::Period {
                spacing: Duration::of(3, TimeUnit::Month),
            },
            Availability::workdays(HourSlot::Range { start: 8, stop: 12 }),
        );
        let json = serde_json::to_string(&sut).unwrap();
        let back: Blueprint = serde_json::from_str(&json).unwrap();
        assert_eq!(sut, back);
    }
}
