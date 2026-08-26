//! Parsing of user-supplied strings for blueprint fields.

use crate::types::Availability;
use crate::types::DayOfWeek;
use crate::types::Duration;
use crate::types::HourSlot;
use crate::types::Priority;
use crate::types::Recurrence;
use crate::types::TimeUnit;
use crate::types::WeekSlot;

/// Parses the `Display` form of a blueprint: `{id} {priority} {recurrence}`
/// `{duration} {availability}`.
///
/// The description is not part of the line format; the caller supplies it.
pub fn blueprint(
    s: &str,
) -> Result<(String, Priority, Recurrence, Duration, Availability), String> {
    const EXAMPLE: &str = "1 NORM ^1d 1h Mon-Fri 08:00-12:00";
    let err = || format!("invalid blueprint '{s}' (expected e.g. '{EXAMPLE}')");
    let mut parts = s.split_whitespace();
    let id = parts.next().ok_or_else(err)?;
    let priority = parts.next().map(priority).transpose()?.ok_or_else(err)?;
    let recurrence = parts.next().map(recurrence).transpose()?.ok_or_else(err)?;
    let duration = parts.next().map(duration).transpose()?.ok_or_else(err)?;
    let availability = availability(parts.collect::<Vec<_>>().as_slice()).ok_or_else(err)??;
    Ok((id.to_string(), priority, recurrence, duration, availability))
}

fn priority(s: &str) -> Result<Priority, String> {
    match s.to_lowercase().as_str() {
        "idle" => Ok(Priority::Idle),
        "norm" => Ok(Priority::Norm),
        "high" => Ok(Priority::High),
        "crit" => Ok(Priority::Crit),
        _ => Err(format!(
            "invalid priority '{s}' (expected idle, norm, high or crit)"
        )),
    }
}

fn day(s: &str) -> Result<DayOfWeek, String> {
    match s.to_lowercase().as_str() {
        "mon" => Ok(DayOfWeek::Mon),
        "tue" => Ok(DayOfWeek::Tue),
        "wed" => Ok(DayOfWeek::Wed),
        "thu" => Ok(DayOfWeek::Thu),
        "fri" => Ok(DayOfWeek::Fri),
        "sat" => Ok(DayOfWeek::Sat),
        "sun" => Ok(DayOfWeek::Sun),
        _ => Err(format!("invalid day '{s}' (expected Mon-Sun)")),
    }
}

fn duration(s: &str) -> Result<Duration, String> {
    for unit in [
        TimeUnit::Second,
        TimeUnit::Minute,
        TimeUnit::Hour,
        TimeUnit::Day,
        TimeUnit::Month,
        TimeUnit::Year,
    ] {
        if let Some(n) = s.strip_suffix(unit.as_str()) {
            let amount: u64 = n
                .parse()
                .map_err(|_| format!("invalid duration '{s}' (expected e.g. 30min, 1h, 1d)"))?;
            return Ok(Duration::of(amount, unit));
        }
    }
    Err(format!(
        "invalid duration '{s}' (expected e.g. 30min, 1h, 1d)"
    ))
}

fn recurrence(s: &str) -> Result<Recurrence, String> {
    match s {
        "once" => Ok(Recurrence::Once),
        "daily" => Ok(Recurrence::Period {
            spacing: Duration::days(1),
        }),
        "weekly" => Ok(Recurrence::Period {
            spacing: Duration::of(7, TimeUnit::Day),
        }),
        "monthly" => Ok(Recurrence::Period {
            spacing: Duration::of(1, TimeUnit::Month),
        }),
        "yearly" => Ok(Recurrence::Period {
            spacing: Duration::of(1, TimeUnit::Year),
        }),
        _ => {
            if let Some(spacing) = s.strip_prefix("every ") {
                return Ok(Recurrence::Period {
                    spacing: duration(spacing)?,
                });
            }
            if let Some(body) = s.strip_prefix('^') {
                return caret(body);
            }
            Err(format!(
                "invalid recurrence '{s}' (expected once, daily, weekly, monthly, yearly, 'every \
                 3mo' or '^{{3,2d}}')"
            ))
        }
    }
}

/// Parses the `Display` form of a recurrence: `^1`, `^{spacing}` or
/// `^{count,spacing}`.
fn caret(s: &str) -> Result<Recurrence, String> {
    if s == "1" {
        return Ok(Recurrence::Once);
    }
    if let Some(body) = s.strip_prefix('{').and_then(|b| b.strip_suffix('}')) {
        let (count, spacing) = body.split_once(',').ok_or_else(|| {
            format!("invalid recurrence '^{{{s}}}' (expected ^{{count,spacing}})")
        })?;
        let count: usize = count
            .parse()
            .map_err(|_| format!("invalid recurrence '^{{{s}}}' (expected ^{{count,spacing}})"))?;
        return Ok(Recurrence::Times {
            count,
            spacing: duration(spacing)?,
        });
    }
    Ok(Recurrence::Period {
        spacing: duration(s)?,
    })
}

fn availability(parts: &[&str]) -> Option<Result<Availability, String>> {
    match parts {
        [hours] if hours.contains(':') => {
            Some(hour_slot(hours).map(|hours| Availability::new(WeekSlot::full(), hours)))
        }
        [days] => Some(week_slot(days).map(Availability::anytime)),
        [days, hours] => Some(
            week_slot(days)
                .and_then(|days| hour_slot(hours).map(|hours| Availability::new(days, hours))),
        ),
        _ => None,
    }
}

fn hour_slot(s: &str) -> Result<HourSlot, String> {
    match s.split_once('-') {
        Some((start, stop)) => Ok(HourSlot::Range {
            start: hour(start)?,
            stop: hour(stop)?,
        }),
        None => Ok(HourSlot::Fixed { hour: hour(s)? }),
    }
}

fn week_slot(s: &str) -> Result<WeekSlot, String> {
    match s.split_once('-') {
        Some((start, stop)) => Ok(WeekSlot::Range {
            start: day(start)?,
            stop: day(stop)?,
        }),
        None => Ok(WeekSlot::Fixed { day: day(s)? }),
    }
}

fn hour(s: &str) -> Result<u32, String> {
    let (h, m) = s
        .split_once(':')
        .ok_or_else(|| format!("invalid hour '{s}' (expected e.g. 08:00)"))?;
    let h: u32 = h
        .parse()
        .map_err(|_| format!("invalid hour '{s}' (expected e.g. 08:00)"))?;
    let m: u32 = m
        .parse()
        .map_err(|_| format!("invalid hour '{s}' (expected e.g. 08:00)"))?;
    if h > 23 || m != 0 {
        return Err(format!("invalid hour '{s}' (expected HH:00 with HH < 24)"));
    }
    Ok(h)
}

#[cfg(test)]
mod test {
    use pretty_assertions::assert_eq;

    use super::*;
    use crate::types::Blueprint;

    #[test]
    fn test_duration() {
        assert_eq!(Ok(Duration::of(5, TimeUnit::Second)), duration("5s"));
        assert_eq!(Ok(Duration::minutes(30)), duration("30min"));
        assert_eq!(Ok(Duration::hours(1)), duration("1h"));
        assert_eq!(Ok(Duration::days(1)), duration("1d"));
        assert_eq!(Ok(Duration::of(3, TimeUnit::Month)), duration("3mo"));
        assert_eq!(Ok(Duration::of(2, TimeUnit::Year)), duration("2y"));
        assert!(duration("3").is_err());
        assert!(duration("abc").is_err());
    }

    #[test]
    fn test_priority() {
        assert_eq!(Ok(Priority::Idle), priority("idle"));
        assert_eq!(Ok(Priority::Norm), priority("NORM"));
        assert_eq!(Ok(Priority::High), priority("high"));
        assert_eq!(Ok(Priority::Crit), priority("crit"));
        assert!(priority("urgent").is_err());
    }

    #[test]
    fn test_recurrence() {
        assert_eq!(Ok(Recurrence::Once), recurrence("once"));
        assert_eq!(
            Ok(Recurrence::Period {
                spacing: Duration::days(1)
            }),
            recurrence("daily")
        );
        assert_eq!(
            Ok(Recurrence::Period {
                spacing: Duration::of(7, TimeUnit::Day)
            }),
            recurrence("weekly")
        );
        assert_eq!(
            Ok(Recurrence::Period {
                spacing: Duration::of(1, TimeUnit::Month)
            }),
            recurrence("monthly")
        );
        assert_eq!(
            Ok(Recurrence::Period {
                spacing: Duration::of(1, TimeUnit::Year)
            }),
            recurrence("yearly")
        );
        assert_eq!(
            Ok(Recurrence::Period {
                spacing: Duration::of(3, TimeUnit::Month)
            }),
            recurrence("every 3mo")
        );
        assert_eq!(Ok(Recurrence::Once), recurrence("^1"));
        assert_eq!(
            Ok(Recurrence::Times {
                count: 3,
                spacing: Duration::days(2)
            }),
            recurrence("^{3,2d}")
        );
        assert_eq!(
            Ok(Recurrence::Period {
                spacing: Duration::of(3, TimeUnit::Year)
            }),
            recurrence("^3y")
        );
        assert!(recurrence("fortnightly").is_err());
        assert!(recurrence("^{3}").is_err());
        assert!(recurrence("^{abc,2d}").is_err());
    }

    #[test]
    fn test_hour_slot() {
        assert_eq!(Ok(HourSlot::Fixed { hour: 8 }), hour_slot("08:00"));
        assert_eq!(
            Ok(HourSlot::Range { start: 8, stop: 12 }),
            hour_slot("08:00-12:00")
        );
        assert!(hour_slot("25:00").is_err());
        assert!(hour_slot("08:30").is_err());
    }

    #[test]
    fn test_week_slot() {
        assert_eq!(
            Ok(WeekSlot::Fixed {
                day: DayOfWeek::Wed
            }),
            week_slot("wed")
        );
        assert_eq!(
            Ok(WeekSlot::Range {
                start: DayOfWeek::Mon,
                stop: DayOfWeek::Fri,
            }),
            week_slot("Mon-Fri")
        );
        assert!(week_slot("foo").is_err());
    }

    #[test]
    fn test_availability() {
        assert_eq!(
            Some(Ok(Availability::new(
                WeekSlot::full(),
                HourSlot::Fixed { hour: 8 }
            ))),
            availability(&["08:00"])
        );
        assert_eq!(
            Some(Ok(Availability::workdays(HourSlot::Range {
                start: 8,
                stop: 12,
            }))),
            availability(&["Mon-Fri", "08:00-12:00"])
        );
        assert_eq!(
            Some(Ok(Availability::anytime(WeekSlot::Fixed {
                day: DayOfWeek::Wed,
            }))),
            availability(&["wed"])
        );
        assert_eq!(None, availability(&[]));
        assert_eq!(None, availability(&["Mon", "08:00", "extra"]));
    }

    #[test]
    fn test_blueprint() {
        assert_eq!(
            Ok((
                "2".to_string(),
                Priority::Norm,
                Recurrence::Period {
                    spacing: Duration::days(1)
                },
                Duration::hours(1),
                Availability::new(WeekSlot::full(), HourSlot::Range { start: 8, stop: 12 }),
            )),
            blueprint("2 NORM ^1d 1h 08:00-12:00")
        );
        assert_eq!(
            Ok((
                "2".to_string(),
                Priority::Norm,
                Recurrence::Period {
                    spacing: Duration::days(1)
                },
                Duration::hours(1),
                Availability::workdays(HourSlot::Range { start: 8, stop: 12 }),
            )),
            blueprint("2 NORM ^1d 1h Mon-Fri 08:00-12:00")
        );
        assert!(blueprint("").is_err());
        assert!(blueprint("2 NORM ^1d 1h").is_err());
        assert!(blueprint("2 NORM ^1d 1h 08:00-12:00 extra").is_err());
        assert!(blueprint("2 URGENT ^1d 1h 08:00-12:00").is_err());
    }

    #[test]
    fn test_roundtrip_blueprint() {
        let suts = [
            Blueprint::from_dsl("2 NORM ^1d 1h 08:00-12:00"),
            Blueprint::from_dsl("7 CRIT ^{3,2d} 30min Mon-Fri"),
            Blueprint::from_dsl("9 HIGH ^1d 2h Mon-Fri 08:00-12:00"),
        ];
        for sut in suts {
            let (id, priority, recurrence, duration, availability) =
                blueprint(&sut.to_string()).unwrap();
            assert_eq!(id, sut.id().to_string());
            assert_eq!(priority, sut.priority());
            assert_eq!(recurrence, sut.recurrence());
            assert_eq!(duration, sut.estimated_duration());
            assert_eq!(availability, sut.availability());
        }
    }

    #[test]
    fn test_roundtrip_duration() {
        let suts = [
            Duration::of(5, TimeUnit::Second),
            Duration::minutes(30),
            Duration::hours(1),
            Duration::days(1),
            Duration::of(3, TimeUnit::Month),
            Duration::of(2, TimeUnit::Year),
        ];
        for sut in suts {
            assert_eq!(Ok(sut), duration(&sut.to_string()));
        }
    }

    #[test]
    fn test_roundtrip_priority() {
        let suts = [
            Priority::Idle,
            Priority::Norm,
            Priority::High,
            Priority::Crit,
        ];
        for sut in suts {
            assert_eq!(Ok(sut), priority(&sut.to_string()));
        }
    }

    #[test]
    fn test_roundtrip_recurrence() {
        let suts = [
            Recurrence::Once,
            Recurrence::Times {
                count: 3,
                spacing: Duration::days(2),
            },
            Recurrence::Period {
                spacing: Duration::of(3, TimeUnit::Month),
            },
        ];
        for sut in suts {
            assert_eq!(Ok(sut), recurrence(&sut.to_string()));
        }
    }

    #[test]
    fn test_roundtrip_availability() {
        let suts = [
            Availability::new(WeekSlot::full(), HourSlot::Fixed { hour: 8 }),
            Availability::new(WeekSlot::full(), HourSlot::Range { start: 8, stop: 12 }),
            Availability::anytime(WeekSlot::Fixed {
                day: DayOfWeek::Wed,
            }),
            Availability::workdays(HourSlot::Range { start: 8, stop: 12 }),
        ];
        for sut in suts {
            let parts = sut.to_string();
            assert_eq!(
                Some(Ok(sut)),
                availability(&parts.split_whitespace().collect::<Vec<_>>())
            );
        }
    }
}
