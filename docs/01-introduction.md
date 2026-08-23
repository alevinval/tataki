# Goal

Tataki helps define, craft, distribute, and manage schedules.

## Problem space

Imagine a yearly dentist visit.

- Medium priority (i.e. nuisance to miss)
- Recurrent event (i.e. a yearly basis)
- Re-scheduled after completion (i.e. next year visit)
- The next recurrence is not exactly 365 days away (i.e. some future events are defined manually)

Or renewing your digital certificate:

- High priority (i.e. better not to miss it)
- One-off event (i.e. certificate valid for 8 years; if the renewal is missed, the chain stops)
- Re-scheduled after completion
- Should be renewed with enough margin (i.e. 3 to 6 months of lead time, should something go wrong)

Or cleaning the VAC filters.

- Low priority (i.e. chore)
- Recurrent event (i.e. twice a year)
- Schedule flexibility (i.e. any weekend within a given month)

The examples above illustrate some of the situations that Tataki must support.

## Representing the domain

The scheduling domain requires primitives to characterize events and their
temporal recurrence characteristics - to name a few:

- Days, Months, Durations, Time units
- Priorities (e.g. Low, Normal, High, Critical)
- Recurrences (e.g. One off, Recurrent, Finite recurrence)
- Slots (e.g. Individual or Ranges of months, days, hours...)

Together, these parameters characterize events in what we call a Blueprint - an
abstract representation of the task - which can be scheduled and instantiated into a
concrete plan or schedule.

A collection of blueprints is kept together as Book, which holds all the representations
for the domain that is being scheduled.
