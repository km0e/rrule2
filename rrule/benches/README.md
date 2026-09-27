# Benchmarks

Criterion benchmarks for parsing, iteration and set operations. They exist
mainly as a **regression baseline**: the iterator core has seen non-trivial
changes (DST duplicate suppression, per-end bound flags, `has_more` peek), and
any future change there should be checked against these numbers first.

## Running

```sh
cargo bench -p rrule2                 # everything
cargo bench -p rrule2 --bench parse   # parsing only
cargo bench -p rrule2 --bench iterate # iteration + set ops only
```

## Comparing before/after a change

```sh
# On the unmodified code (e.g. main):
cargo bench -- --save-baseline pre-change

# After your change:
cargo bench -- --baseline pre-change
```

Criterion prints per-benchmark deltas; anything beyond noise (±2–3%) in
`iterate/*` deserves a look. Plots and raw data land in `target/criterion/`.

## What each benchmark guards

| Benchmark | Guards |
|---|---|
| `parse/rrule_only` | Minimal `DTSTART`+`RRULE` parse |
| `parse/with_tzid` | `TZID` parameter handling incl. IANA timezone lookup |
| `parse/full_set` | Realistic set: RRULE + RDATE + EXDATE |
| `parse/full_set_lowercase` | Case-insensitive paths (RFC 5545 §3.1); must track `full_set` closely |
| `iterate/daily_count_1000` | Steady-state daily iteration |
| `iterate/weekly_byday_count_1000` | `BYDAY` filtering |
| `iterate/monthly_bymonthday_count_500` | `BYMONTHDAY` expansion |
| `iterate/yearly_bymonth_bymonthday_count_100` | Yearly steps (highest per-step cost) |
| `iterate/hourly_dst_london_count_1500` | Crosses the 2024-03-31 spring-forward in Europe/London; exercises the duplicate-instant suppression path |
| `iterate/mixed_set_rdate_exdate_count_1000` | Merged RRULE/RDATE iteration with per-date exclusion checks |
| `iterate/daily_until_all_unchecked_365` | Unbounded collection stopped by a bound (no `COUNT`) |
| `ops/contains_hit_middle_of_1000` | Early-exit membership walk (half the set) |
| `ops/contains_miss_beyond_end_1000` | Full membership walk on a miss |
| `ops/to_string_full_set` | Serialization incl. RDATE/EXDATE UTC conversion |

## Baseline (2026-09-27)

Intel Core i5-10200H @ 2.40GHz (8 threads), rustc 1.98.1, criterion 0.7,
`--all-features` equivalent code (benches build with default features + bench
profile; no feature-gated code is exercised).

| Benchmark | mean |
|---|---:|
| `parse/rrule_only` | 1.54 µs |
| `parse/with_tzid` | 1.84 µs |
| `parse/full_set` | 5.18 µs |
| `parse/full_set_lowercase` | 5.17 µs |
| `iterate/daily_count_1000` | 178.6 µs |
| `iterate/weekly_byday_count_1000` | 226.7 µs |
| `iterate/monthly_bymonthday_count_500` | 193.5 µs |
| `iterate/yearly_bymonth_bymonthday_count_100` | 200.8 µs |
| `iterate/hourly_dst_london_count_1500` | 402.0 µs |
| `iterate/mixed_set_rdate_exdate_count_1000` | 187.1 µs |
| `iterate/daily_until_all_unchecked_365` | 66.4 µs |
| `ops/contains_hit_middle_of_1000` | 91.4 µs |
| `ops/contains_miss_beyond_end_1000` | 67.2 µs |
| `ops/to_string_full_set` | 2.73 µs |

Rough per-occurrence cost of iteration is ~180–270 ns across the rule shapes
above. Note these are single-run reference numbers, not a CI-tracked
statistic; rerun on your own machine before drawing conclusions from small
deltas.
