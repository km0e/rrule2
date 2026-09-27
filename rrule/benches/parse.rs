//! Benchmarks for parsing iCalendar recurrence strings into [`RRuleSet`]s.
//!
//! Run with `cargo bench -p rrule2 --bench parse` (see `benches/README.md`).

use std::str::FromStr;

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use rrule2::RRuleSet;

const RRULE_ONLY: &str = "DTSTART:20240101T090000Z\nRRULE:FREQ=DAILY;COUNT=100";

const WITH_TZID: &str = "DTSTART;TZID=Europe/Berlin:20240101T090000\nRRULE:FREQ=DAILY;COUNT=100";

const FULL_SET: &str = "DTSTART:20240101T090000Z\n\
    RRULE:FREQ=DAILY;COUNT=100\n\
    RDATE:20240201T090000Z,20240202T090000Z,20240301T090000Z,20240401T090000Z,20240501T090000Z\n\
    EXDATE:20240110T090000Z,20240120T090000Z,20240130T090000Z";

/// Same rule as `FULL_SET` but with lowercase names/values, to surface the
/// cost of the case-insensitive parsing paths (RFC 5545 §3.1 support).
const FULL_SET_LOWERCASE: &str = "dtstart:20240101t090000z\n\
    rrule:freq=daily;count=100\n\
    rdate:20240201t090000z,20240202t090000z,20240301t090000z,20240401t090000z,20240501t090000z\n\
    exdate:20240110t090000z,20240120t090000z,20240130t090000z";

fn bench_parse(c: &mut Criterion) {
    let mut group = c.benchmark_group("parse");

    group.bench_function("rrule_only", |b| {
        b.iter(|| RRuleSet::from_str(black_box(RRULE_ONLY)).unwrap())
    });

    // Includes the IANA timezone lookup for the `TZID` parameter.
    group.bench_function("with_tzid", |b| {
        b.iter(|| RRuleSet::from_str(black_box(WITH_TZID)).unwrap())
    });

    // A realistic set: one RRULE, five RDATEs, three EXDATEs.
    group.bench_function("full_set", |b| {
        b.iter(|| RRuleSet::from_str(black_box(FULL_SET)).unwrap())
    });

    group.bench_function("full_set_lowercase", |b| {
        b.iter(|| RRuleSet::from_str(black_box(FULL_SET_LOWERCASE)).unwrap())
    });

    group.finish();
}

criterion_group!(benches, bench_parse);
criterion_main!(benches);
