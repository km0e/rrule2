use crate::{tests::common::check_occurrences, RRuleSet};

#[test]
fn daylight_savings_1() {
    let rrule: RRuleSet =
        "DTSTART;TZID=America/Vancouver:20210301T022210\nRRULE:FREQ=DAILY;COUNT=30"
            .parse()
            .unwrap();

    let dates = rrule.all_unchecked();
    check_occurrences(
        &dates,
        &[
            "2021-03-01T02:22:10-08:00",
            "2021-03-02T02:22:10-08:00",
            "2021-03-03T02:22:10-08:00",
            "2021-03-04T02:22:10-08:00",
            "2021-03-05T02:22:10-08:00",
            "2021-03-06T02:22:10-08:00",
            "2021-03-07T02:22:10-08:00",
            "2021-03-08T02:22:10-08:00",
            "2021-03-09T02:22:10-08:00",
            "2021-03-10T02:22:10-08:00",
            "2021-03-11T02:22:10-08:00",
            "2021-03-12T02:22:10-08:00",
            "2021-03-13T02:22:10-08:00",
            "2021-03-14T03:22:10-07:00",
            "2021-03-15T02:22:10-07:00",
            "2021-03-16T02:22:10-07:00",
            "2021-03-17T02:22:10-07:00",
            "2021-03-18T02:22:10-07:00",
            "2021-03-19T02:22:10-07:00",
            "2021-03-20T02:22:10-07:00",
            "2021-03-21T02:22:10-07:00",
            "2021-03-22T02:22:10-07:00",
            "2021-03-23T02:22:10-07:00",
            "2021-03-24T02:22:10-07:00",
            "2021-03-25T02:22:10-07:00",
            "2021-03-26T02:22:10-07:00",
            "2021-03-27T02:22:10-07:00",
            "2021-03-28T02:22:10-07:00",
            "2021-03-29T02:22:10-07:00",
            "2021-03-30T02:22:10-07:00",
        ],
    );
}

#[test]
fn daylight_savings_2() {
    let dates = "DTSTART;TZID=Europe/Paris:20210214T093000\n\
        RRULE:FREQ=WEEKLY;UNTIL=20210508T083000Z;INTERVAL=2;BYDAY=MO;WKST=MO"
        .parse::<RRuleSet>()
        .unwrap()
        .all(u16::MAX)
        .dates;
    check_occurrences(
        &dates,
        &[
            "2021-02-22T09:30:00+01:00",
            "2021-03-08T09:30:00+01:00",
            "2021-03-22T09:30:00+01:00",
            "2021-04-05T09:30:00+02:00", // Switching to daylight saving time.
            "2021-04-19T09:30:00+02:00",
            "2021-05-03T09:30:00+02:00",
        ],
    );
}

#[test]
fn daylight_savings_dtstart_in_gap() {
    // Regression test for fmeringdal/rust-rrule#109/#115: a DTSTART that
    // falls in a DST gap (the local time does not exist) must be parsed
    // instead of rejected. Per RFC 5545 §3.3.5 it is interpreted using the
    // UTC offset before the gap, so `20240331T013000` in Europe/London
    // (clocks jump 01:00 GMT -> 02:00 BST) becomes 01:30 UTC = 02:30 BST.
    let dates = "DTSTART;TZID=Europe/London:20240331T013000\n\
        RRULE:FREQ=DAILY;COUNT=3"
        .parse::<RRuleSet>()
        .unwrap()
        .all(u16::MAX)
        .dates;
    check_occurrences(
        &dates,
        &[
            "2024-03-31T02:30:00+01:00",
            "2024-04-01T02:30:00+01:00",
            "2024-04-02T02:30:00+01:00",
        ],
    );
}

#[test]
fn daylight_savings_dtstart_ambiguous() {
    // Regression test for fmeringdal/rust-rrule#109/#115: a DTSTART that
    // occurs twice (when the clocks go back) must be parsed instead of
    // rejected. Per RFC 5545 §3.3.5 it is interpreted using the UTC offset
    // before the transition, so `20241027T013000` in Europe/London (clocks
    // fall back 02:00 BST -> 01:00 GMT) is the earlier occurrence, 01:30 BST.
    let dates = "DTSTART;TZID=Europe/London:20241027T013000\n\
        RRULE:FREQ=DAILY;COUNT=3"
        .parse::<RRuleSet>()
        .unwrap()
        .all(u16::MAX)
        .dates;
    check_occurrences(
        &dates,
        &[
            "2024-10-27T01:30:00+01:00",
            "2024-10-28T01:30:00+00:00",
            "2024-10-29T01:30:00+00:00",
        ],
    );
}

#[test]
fn daylight_savings_rdate_in_gap() {
    // RDATE/EXDATE values go through the same resolution as DTSTART.
    let dates = "DTSTART;TZID=Europe/London:20240401T093000\n\
        RDATE;TZID=Europe/London:20250330T013000"
        .parse::<RRuleSet>()
        .unwrap()
        .all(u16::MAX)
        .dates;
    check_occurrences(
        &dates,
        &["2025-03-30T02:30:00+01:00"], // RDATE, resolved out of the gap.
    );
}
