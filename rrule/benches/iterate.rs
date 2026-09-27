//! Benchmarks for iterating recurrence sets and set-level operations.
//!
//! All sets are parsed once in the setup phase (not timed); each timed
//! iteration clones the set, mirroring how `RRuleSet::all` consumes it.
//!
//! Run with `cargo bench -p rrule2 --bench iterate` (see `benches/README.md`).

use std::str::FromStr;

use chrono::TimeZone;
use criterion::{black_box, criterion_group, criterion_main, BatchSize, Criterion};
use rrule2::{RRuleSet, Tz};

fn parse(input: &str) -> RRuleSet {
    RRuleSet::from_str(input).unwrap()
}

fn bench_iterate(c: &mut Criterion) {
    let mut group = c.benchmark_group("iterate");

    let daily = parse("DTSTART:20240101T090000Z\nRRULE:FREQ=DAILY;COUNT=1000");
    group.bench_function("daily_count_1000", |b| {
        b.iter_batched(
            || daily.clone(),
            |set| black_box(set.all(1000).dates.len()),
            BatchSize::SmallInput,
        )
    });

    let weekly = parse("DTSTART:20240101T090000Z\nRRULE:FREQ=WEEKLY;BYDAY=MO,WE;COUNT=1000");
    group.bench_function("weekly_byday_count_1000", |b| {
        b.iter_batched(
            || weekly.clone(),
            |set| black_box(set.all(1000).dates.len()),
            BatchSize::SmallInput,
        )
    });

    let monthly = parse("DTSTART:20240101T090000Z\nRRULE:FREQ=MONTHLY;BYMONTHDAY=1,15;COUNT=500");
    group.bench_function("monthly_bymonthday_count_500", |b| {
        b.iter_batched(
            || monthly.clone(),
            |set| black_box(set.all(500).dates.len()),
            BatchSize::SmallInput,
        )
    });

    let yearly =
        parse("DTSTART:20240101T090000Z\nRRULE:FREQ=YEARLY;BYMONTH=6;BYMONTHDAY=15;COUNT=100");
    group.bench_function("yearly_bymonth_bymonthday_count_100", |b| {
        b.iter_batched(
            || yearly.clone(),
            |set| black_box(set.all(100).dates.len()),
            BatchSize::SmallInput,
        )
    });

    // Crosses the 2024-03-31 spring-forward transition in Europe/London, so
    // every run exercises the duplicate-instant suppression path (#115).
    let hourly_dst = parse(
        "DTSTART;TZID=Europe/London:20240101T090000\nRRULE:FREQ=HOURLY;COUNT=1500",
    );
    group.bench_function("hourly_dst_london_count_1500", |b| {
        b.iter_batched(
            || hourly_dst.clone(),
            |set| black_box(set.all(1500).dates.len()),
            BatchSize::SmallInput,
        )
    });

    // A set mixing RRULE with RDATE/EXDATE: exercises the merged iteration
    // and the per-date exclusion checks.
    let mixed = parse(
        "DTSTART:20240101T090000Z\n\
         RRULE:FREQ=DAILY;COUNT=1000\n\
         RDATE:20240205T090000Z,20240206T090000Z,20240305T090000Z,20240405T090000Z\n\
         EXDATE:20240110T090000Z,20240120T090000Z,20240130T090000Z,20240210T090000Z",
    );
    group.bench_function("mixed_set_rdate_exdate_count_1000", |b| {
        b.iter_batched(
            || mixed.clone(),
            |set| black_box(set.all(1000).dates.len()),
            BatchSize::SmallInput,
        )
    });

    // Unbounded collection stopped by a `before` bound instead of `COUNT`.
    let until = parse("DTSTART:20240101T090000Z\nRRULE:FREQ=DAILY;UNTIL=20241231T090000Z");
    group.bench_function("daily_until_all_unchecked_365", |b| {
        b.iter_batched(
            || until.clone(),
            |set| black_box(set.all_unchecked().len()),
            BatchSize::SmallInput,
        )
    });

    group.finish();
}

fn bench_ops(c: &mut Criterion) {
    let mut group = c.benchmark_group("ops");

    let daily = parse("DTSTART:20240101T090000Z\nRRULE:FREQ=DAILY;COUNT=1000");

    // Membership hit in the middle: walks half the set before stopping.
    let mid = daily.clone().all(500).dates.last().copied().unwrap();
    group.bench_function("contains_hit_middle_of_1000", |b| {
        b.iter(|| black_box(daily.contains(mid)))
    });

    // Membership miss past the end: walks the entire set.
    let beyond = Tz::UTC.with_ymd_and_hms(2025, 1, 1, 9, 0, 0).unwrap();
    group.bench_function("contains_miss_beyond_end_1000", |b| {
        b.iter(|| black_box(daily.contains(beyond)))
    });

    // Serialization of a set with RRULE/RDATE/EXDATE lines.
    let mixed = parse(
        "DTSTART:20240101T090000Z\n\
         RRULE:FREQ=DAILY;COUNT=1000\n\
         RDATE:20240205T090000Z,20240206T090000Z\n\
         EXDATE:20240110T090000Z,20240120T090000Z",
    );
    group.bench_function("to_string_full_set", |b| {
        b.iter(|| black_box(mixed.to_string()))
    });

    group.finish();
}

criterion_group!(benches, bench_iterate, bench_ops);
criterion_main!(benches);
