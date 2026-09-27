<h1 align="center">RRule.rs</h1>
<p align="center">A pure and efficient Rust implementation of recurrence rules as defined in the iCalendar RFC.</p>
<p align="center">
  <a href="https://crates.io/crates/rrule2"><img src="https://img.shields.io/crates/v/rrule2.svg" /></a>
  <a href="https://docs.rs/rrule2/latest/rrule2/"><img src="https://img.shields.io/badge/docs-rrule2-blue" /></a>
</p>

> **Fork notice**: `rrule2` is a maintained fork of the excellent [`rrule` crate](https://crates.io/crates/rrule) by [fmeringdal/rust-rrule](https://github.com/fmeringdal/rust-rrule), published as a separate crate to ship additional fixes.
> API is unchanged relative to `rrule` 0.14.0 — see the [CHANGELOG](CHANGELOG.md) for what is fixed.

## Specification

This crate follows the [iCalendar (RFC-5545) specification][ical_spec] for the "Recurrence Rule".
The Recurrence Rule spec corresponds to the `RRule` object in this crate.
In addition, it allows for adding the ["DTSTART" property][dtstart_property] separated by a newline.

The crate allows for a "BYEASTER" filter. But this is opt-in with the feature flag `"by-easter"`.

### RRuleSet

`RRuleSet` allows for a combination for `RRule`s and some other properties.

- List of [RRules](https://icalendar.org/iCalendar-RFC-5545/3-8-5-3-recurrence-rule.html):
  Allows multiple RRules to be combined. (Union, `A ∪ B`)
- List of [RDates](https://icalendar.org/iCalendar-RFC-5545/3-8-5-2-recurrence-date-times.html):
  A list of datetime combinations to always include. (Union, `A ∪ B`)
- List of [ExRule](https://datatracker.ietf.org/doc/html/rfc2445#section-4.8.5.2) (see note below):
  Allows of RRules that are removed from the results. (Complement `A \ B` or `A - B`)
- List of [ExDate](https://icalendar.org/iCalendar-RFC-5545/3-8-5-1-exception-date-times.html):
  A list of datetime combinations to always exclude. (Complement `A \ B` or `A - B`)

Note: "EXRULE" was originally part of [RFC 2445](https://datatracker.ietf.org/doc/html/rfc2445#section-4.8.5.2),
[RFC 5545][ical_spec] obsoletes this specification.
But "EXRULE" works exactly the same als "RRULE" except that it excludes dates. You can enable "EXRULE" by enabling the "exrule" feature flag which is disabled by default.

If you notice that the implementation differs from the specifications above, please open an issue.

## Library Usage

```rust
use rrule2::RRuleSet;

// RRule that starts 2012.02.01 and occurs daily for 3 days.
let rrule: RRuleSet = "DTSTART:20120201T093000Z\nRRULE:FREQ=DAILY;COUNT=3".parse().unwrap();

// Set hard limit in case of infinitely recurring rules.
let limit = 100;
// Get all recurrences of the rrule
let result = rrule.all(limit);
assert_eq!(result.dates.len(), 3);
```

See more examples at [docs.rs](https://docs.rs/rrule2)

## Command Line Tool Usage

Install the command line tool with:

```bash
cargo install rrule2 --features="cli-tool"
```

Then run it with:

```bash
rrule2 "DTSTART:20120201T093000Z\nRRULE:FREQ=DAILY;COUNT=3"
```

## Security

You should read the [security docs](https://github.com/km0e/rrule2/blob/main/SECURITY.md) if you use arbitrary inputs from users for constructing the recurrence rules.

## Limitation and limits

All dates are limited to the range or years +/-262_000[^1] because of [Chrono][chrono] limits.
See [Chrono's limits for more info](https://github.com/chronotope/chrono#limitations).

Supported timezones are limited to by the timezones that [Chrono-Tz][chrono-tz] supports.
This is equivalent to the IANA database.
See [Chrono-Tz's limits for more info](https://github.com/chronotope/chrono-tz/#limiting-the-timezone-table-to-zones-of-interest).

### Validation Limits

<a name="validation_limits"></a>
Because the specifications do give a lot of flexibilities this can be [abused very easily](#Security).
In order to prevent most of the abuse we've imposed arbitrary limitation when on the `RRuleSet::all`
method. The validation limits aren't enforced for the `RRuleSet::all_unchecked` method or when
using the `Iterator` api directly.

Limitations:
| Description | Arbitrary Limit | Crate Limit |
|----------------------------------|-----------------------|-----------------------------|
| Year range | -10_000..=10_000 | -262_000..=262_000 (Chrono) |
| Max interval with freq Yearly | 10_000 (10000 years) | 65_535 (u16::MAX) |
| Max interval with freq Monthly | 1_000 (~83 years) | 65_535 (u16::MAX) |
| Max interval with freq Weekly | 1_000 (~19 years) | 65_535 (u16::MAX) |
| Max interval with freq Daily | 10_000 (~27 years) | 65_535 (u16::MAX) |
| Max interval with freq Hourly | 10_000 (~416 days) | 65_535 (u16::MAX) |
| Max interval with freq Minutely | 10_000 (~7 days) | 65_535 (u16::MAX) |
| Max interval with freq Secondly | 50_000 (~13 hours) | 65_535 (u16::MAX) |
| Iteration limit | 100_000 | 4_294_967_295 (u32::MAX) |

By default, the "Arbitrary Limit" is used. If you instead want to use the "Crate Limit".
Make sure you [understand the risks that come with this](#safety).

## Behavior notes

Notes on edge cases of RFC 5545 that come up regularly.

### `DTSTART` and the first occurrence

RFC 5545 recommends ("SHOULD") that `DTSTART` is synchronized with the recurrence
rule (e.g. a `BYDAY=WE` rule should start on a Wednesday). If it is not, the
standard leaves the result **undefined**. This crate generates only the
occurrences produced by the rule: an unsynchronized `DTSTART` is not force-added
as a first instance, and `COUNT` counts the rule's occurrences.

Google Calendar behaves differently: it always emits the `DTSTART` as the first
instance, even when it does not match the rule (and even when `COUNT` then
produces one more event than requested). To reproduce Google's behavior, list
the `DTSTART` as an `RDATE` explicitly:

```
DTSTART;TZID=Europe/Berlin:20240530T200000
RDATE;TZID=Europe/Berlin:20240530T200000
RRULE:FREQ=WEEKLY;COUNT=3;BYDAY=WE
```

This is safe even when `DTSTART` *does* match the rule: duplicate instants are
removed from the recurrence set (RFC 5545 §3.8.5.3).

References: [fmeringdal/rust-rrule#119](https://github.com/fmeringdal/rust-rrule/issues/119),
[RFC 5545 §3.8.5.3](https://icalendar.org/iCalendar-RFC-5545/3-8-5-3-recurrence-rule.html).

### `FREQ=YEARLY` without `BYMONTH`

With `FREQ=YEARLY`, a `BYMONTHDAY` (or `BYDAY`, ...) without `BYMONTH` expands
within **every month of the year**: `RRULE:FREQ=YEARLY;BYMONTHDAY=20` means
"the 20th of every month, every year", not "yearly on the 20th". This follows
the expansion semantics of RFC 5545 and matches python-dateutil and librrule.

To get a yearly occurrence on a specific day, name the month:

```
RRULE:FREQ=YEARLY;BYMONTH=4;BYMONTHDAY=20   → yearly on April 20
```

Reference: [fmeringdal/rust-rrule#127](https://github.com/fmeringdal/rust-rrule/issues/127).

### Daylight saving time

Occurrences follow the wall clock of the `DTSTART` timezone. When an occurrence
lands on a DST transition:

- A time that does not exist (spring-forward gap) is interpreted using the UTC
  offset **before** the gap, per RFC 5545 §3.3.5 (e.g. `01:30` on the day
  Europe/London jumps to BST is generated as `01:30 GMT` = `02:30 BST`).
- A time that occurs twice (fall-back) is generated once, using the offset
  before the transition (the earlier instant).
- Duplicate instants are never emitted, and `COUNT` counts distinct
  occurrences.

The same rules apply to parsing `DTSTART`, `RDATE` and `EXDATE` values on
transition days.

References: [fmeringdal/rust-rrule#109](https://github.com/fmeringdal/rust-rrule/issues/109),
[fmeringdal/rust-rrule#115](https://github.com/fmeringdal/rust-rrule/issues/115).

## Inspired by

- [python-dateutil library](http://labix.org/python-dateutil/)
- [rrule.js](https://github.com/jakubroztocil/rrule)

## License

The code in this project is licensed under the [MIT](LICENSE-MIT) or [Apache 2.0](LICENSE-APACHE) license.

All contributions to this project will be similarly licensed.

[chrono]: https://github.com/chronotope/chrono
[chrono-tz]: https://github.com/chronotope/chrono-tz/
[ical_spec]: https://icalendar.org/iCalendar-RFC-5545/3-3-10-recurrence-rule.html
[dtstart_property]: https://icalendar.org/iCalendar-RFC-5545/3-8-2-4-date-time-start.html

[^1]: See [validation limits](#validation_limits) sections more info.
