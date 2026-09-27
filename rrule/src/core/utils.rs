use crate::RRuleResult;
use crate::{iter::rrule_iter::WasLimited, Tz};
use std::ops::{
    Bound::{Excluded, Included, Unbounded},
    RangeBounds,
};

/// Helper function to collect dates given some filters.
///
/// In the case where the iterator ended with errors, the error will be included,
/// otherwise the second value of the return tuple will be `None`.
pub(super) fn collect_with_error<T>(
    mut iterator: T,
    start: &Option<chrono::DateTime<Tz>>,
    end: &Option<chrono::DateTime<Tz>>,
    inclusive_start: bool,
    inclusive_end: bool,
    limit: Option<u16>,
) -> RRuleResult
where
    T: Iterator<Item = chrono::DateTime<Tz>> + WasLimited,
{
    let mut list = vec![];
    let mut was_limited = false;
    let mut has_more = false;

    loop {
        // The caller's cap is reached. Check whether the rule can still
        // produce another occurrence, to distinguish "truncated at the cap"
        // from "the rule legitimately ends here" (e.g. `COUNT` exhausted at
        // exactly `limit`). Computing one extra occurrence is cheap and exact.
        if matches!(limit, Some(limit) if usize::from(limit) == list.len()) {
            has_more = iterator.next().is_some();
            if !has_more {
                // The iterator ran out on its own. If it stopped because of
                // the internal iteration safety limit (instead of natural
                // exhaustion), surface that through `limited`.
                was_limited = iterator.was_limited();
            }
            break;
        }

        match iterator.next() {
            Some(value) => {
                if is_in_range(&value, start, end, inclusive_start, inclusive_end) {
                    list.push(value);
                }
                if has_reached_the_end(&value, end, inclusive_end) {
                    // Date is after end date, so can stop iterating
                    break;
                }
            }
            None => {
                was_limited = iterator.was_limited();
                break;
            }
        }
    }

    RRuleResult {
        dates: list,
        limited: was_limited,
        has_more,
    }
}

/// Checks if `date` is after `end`.
fn has_reached_the_end(
    date: &chrono::DateTime<Tz>,
    end: &Option<chrono::DateTime<Tz>>,
    inclusive_end: bool,
) -> bool {
    match end {
        Some(end) if inclusive_end => date > end,
        Some(end) => date >= end,
        None => false,
    }
}

/// Helper function to determine if a date is within a given range.
pub(super) fn is_in_range(
    date: &chrono::DateTime<Tz>,
    start: &Option<chrono::DateTime<Tz>>,
    end: &Option<chrono::DateTime<Tz>>,
    inclusive_start: bool,
    inclusive_end: bool,
) -> bool {
    let start_bound = match (start, inclusive_start) {
        (Some(start), true) => Included(*start),
        (Some(start), false) => Excluded(*start),
        (None, _) => Unbounded,
    };
    let end_bound = match (end, inclusive_end) {
        (Some(end), true) => Included(*end),
        (Some(end), false) => Excluded(*end),
        (None, _) => Unbounded,
    };
    (start_bound, end_bound).contains(date)
}

#[cfg(test)]
mod tests {
    use crate::core::Tz;

    use super::*;
    use chrono::TimeZone;

    const UTC: Tz = Tz::UTC;

    #[test]
    fn in_range_exclusive_start_to_end() {
        let inclusive = false;
        let start = UTC.with_ymd_and_hms(2021, 10, 1, 8, 0, 0).unwrap();
        let end = UTC.with_ymd_and_hms(2021, 10, 1, 10, 0, 0).unwrap();

        // In middle
        assert!(is_in_range(
            &UTC.with_ymd_and_hms(2021, 10, 1, 9, 0, 0).unwrap(),
            &Some(start),
            &Some(end),
            inclusive,
            inclusive,
        ));
        // To small
        assert!(!is_in_range(
            &UTC.with_ymd_and_hms(2021, 10, 1, 7, 0, 0).unwrap(),
            &Some(start),
            &Some(end),
            inclusive,
            inclusive,
        ));
        // To big
        assert!(!is_in_range(
            &UTC.with_ymd_and_hms(2021, 10, 1, 11, 0, 0).unwrap(),
            &Some(start),
            &Some(end),
            inclusive,
            inclusive,
        ));
        // Equal to end
        assert!(!is_in_range(&end, &Some(start), &Some(end), inclusive, inclusive));
        // Equal to start
        assert!(!is_in_range(&start, &Some(start), &Some(end), inclusive, inclusive));
    }

    #[test]
    fn in_range_exclusive_start() {
        let inclusive = false;
        let start = UTC.with_ymd_and_hms(2021, 10, 1, 8, 0, 0).unwrap();

        // Just after
        assert!(is_in_range(
            &UTC.with_ymd_and_hms(2021, 10, 1, 9, 0, 0).unwrap(),
            &Some(start),
            &None,
            inclusive,
            inclusive,
        ));
        // To small
        assert!(!is_in_range(
            &UTC.with_ymd_and_hms(2021, 10, 1, 7, 0, 0).unwrap(),
            &Some(start),
            &None,
            inclusive,
            inclusive,
        ));
        // Bigger
        assert!(is_in_range(
            &UTC.with_ymd_and_hms(2021, 10, 2, 8, 0, 0).unwrap(),
            &Some(start),
            &None,
            inclusive,
            inclusive,
        ));
        // Equal to start
        assert!(!is_in_range(&start, &Some(start), &None, inclusive, inclusive));
    }

    #[test]
    fn in_range_exclusive_end() {
        let inclusive = false;
        let end = UTC.with_ymd_and_hms(2021, 10, 1, 10, 0, 0).unwrap();

        // Just before
        assert!(is_in_range(
            &UTC.with_ymd_and_hms(2021, 10, 1, 9, 0, 0).unwrap(),
            &None,
            &Some(end),
            inclusive,
            inclusive,
        ));
        // Smaller
        assert!(is_in_range(
            &UTC.with_ymd_and_hms(2021, 9, 20, 10, 0, 0).unwrap(),
            &None,
            &Some(end),
            inclusive,
            inclusive,
        ));
        // Bigger
        assert!(!is_in_range(
            &UTC.with_ymd_and_hms(2021, 10, 2, 8, 0, 0).unwrap(),
            &None,
            &Some(end),
            inclusive,
            inclusive,
        ));
        // Equal to end
        assert!(!is_in_range(&end, &None, &Some(end), inclusive, inclusive));
    }

    #[test]
    fn in_range_exclusive_all() {
        let inclusive = false;

        // Some date
        assert!(is_in_range(
            &UTC.with_ymd_and_hms(2021, 10, 1, 9, 0, 0).unwrap(),
            &None,
            &None,
            inclusive,
            inclusive,
        ));
        // Smaller
        assert!(is_in_range(
            &UTC.with_ymd_and_hms(2021, 9, 20, 10, 0, 0).unwrap(),
            &None,
            &None,
            inclusive,
            inclusive,
        ));
        // Bigger
        assert!(is_in_range(
            &UTC.with_ymd_and_hms(2021, 10, 2, 8, 0, 0).unwrap(),
            &None,
            &None,
            inclusive,
            inclusive,
        ));
    }

    // ---------------- inclusive -----------------------

    #[test]
    fn in_range_inclusive_start_to_end() {
        let inclusive = true;
        let start = UTC.with_ymd_and_hms(2021, 10, 1, 8, 0, 0).unwrap();
        let end = UTC.with_ymd_and_hms(2021, 10, 1, 10, 0, 0).unwrap();

        // In middle
        assert!(is_in_range(
            &UTC.with_ymd_and_hms(2021, 10, 1, 9, 0, 0).unwrap(),
            &Some(start),
            &Some(end),
            inclusive,
            inclusive,
        ));
        // To small
        assert!(!is_in_range(
            &UTC.with_ymd_and_hms(2021, 10, 1, 7, 0, 0).unwrap(),
            &Some(start),
            &Some(end),
            inclusive,
            inclusive,
        ));
        // To big
        assert!(!is_in_range(
            &UTC.with_ymd_and_hms(2021, 10, 1, 11, 0, 0).unwrap(),
            &Some(start),
            &Some(end),
            inclusive,
            inclusive,
        ));
        // Equal to end
        assert!(is_in_range(&end, &Some(start), &Some(end), inclusive, inclusive));
        // Equal to start
        assert!(is_in_range(&start, &Some(start), &Some(end), inclusive, inclusive));
    }

    #[test]
    fn in_range_inclusive_start() {
        let inclusive = true;
        let start = UTC.with_ymd_and_hms(2021, 10, 1, 8, 0, 0).unwrap();

        // Just after
        assert!(is_in_range(
            &UTC.with_ymd_and_hms(2021, 10, 1, 9, 0, 0).unwrap(),
            &Some(start),
            &None,
            inclusive,
            inclusive,
        ));
        // To small
        assert!(!is_in_range(
            &UTC.with_ymd_and_hms(2021, 10, 1, 7, 0, 0).unwrap(),
            &Some(start),
            &None,
            inclusive,
            inclusive,
        ));
        // Bigger
        assert!(is_in_range(
            &UTC.with_ymd_and_hms(2021, 10, 2, 8, 0, 0).unwrap(),
            &Some(start),
            &None,
            inclusive,
            inclusive,
        ));
        // Equal to start
        assert!(is_in_range(&start, &Some(start), &None, inclusive, inclusive));
    }

    #[test]
    fn in_range_inclusive_end() {
        let inclusive = true;
        let end = UTC.with_ymd_and_hms(2021, 10, 1, 10, 0, 0).unwrap();

        // Just before
        assert!(is_in_range(
            &UTC.with_ymd_and_hms(2021, 10, 1, 9, 0, 0).unwrap(),
            &None,
            &Some(end),
            inclusive,
            inclusive,
        ));
        // Smaller
        assert!(is_in_range(
            &UTC.with_ymd_and_hms(2021, 9, 20, 10, 0, 0).unwrap(),
            &None,
            &Some(end),
            inclusive,
            inclusive,
        ));
        // Bigger
        assert!(!is_in_range(
            &UTC.with_ymd_and_hms(2021, 10, 2, 8, 0, 0).unwrap(),
            &None,
            &Some(end),
            inclusive,
            inclusive,
        ));
        // Equal to end
        assert!(is_in_range(&end, &None, &Some(end), inclusive, inclusive));
    }

    #[test]
    fn in_range_inclusive_all() {
        let inclusive = true;

        // Some date
        assert!(is_in_range(
            &UTC.with_ymd_and_hms(2021, 10, 1, 9, 0, 0).unwrap(),
            &None,
            &None,
            inclusive,
            inclusive,
        ));
        // Smaller
        assert!(is_in_range(
            &UTC.with_ymd_and_hms(2021, 9, 20, 10, 0, 0).unwrap(),
            &None,
            &None,
            inclusive,
            inclusive,
        ));
        // Bigger
        assert!(is_in_range(
            &UTC.with_ymd_and_hms(2021, 10, 2, 8, 0, 0).unwrap(),
            &None,
            &None,
            inclusive,
            inclusive,
        ));
    }
}
