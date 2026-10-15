//! Czech time (Europe/Prague). The database stores UTC;
//! times are shown, and days and months are counted, in Czech time.
//!
//! Czechia follows the EU summer time rule: UTC+2 from 01:00 UTC on the
//! last Sunday of March until 01:00 UTC on the last Sunday of October,
//! UTC+1 the rest of the year. If the EU ever drops the clock change, this
//! is the one place to change.

use chrono::{Datelike, Duration, NaiveDate, NaiveDateTime, NaiveTime, Utc, Weekday};

/// The current Czech date and time.
pub fn now() -> NaiveDateTime {
    from_utc(Utc::now().naive_utc())
}

/// Today's date in Czechia (it changes at midnight Czech time, not UTC).
pub fn today() -> NaiveDate {
    now().date()
}

/// A stored (UTC) time as Czech time.
pub fn from_utc(utc: NaiveDateTime) -> NaiveDateTime {
    utc + Duration::hours(offset_hours(utc))
}

/// A Czech time as UTC, to compare with stored times. Meant for times
/// outside the hour the clocks change (like midnight).
pub fn to_utc(local: NaiveDateTime) -> NaiveDateTime {
    let guess = local - Duration::hours(1);
    local - Duration::hours(offset_hours(guess))
}

/// 2 in summer time, 1 otherwise.
fn offset_hours(utc: NaiveDateTime) -> i64 {
    let year = utc.year();
    let at_one = |date: NaiveDate| date.and_time(NaiveTime::from_hms_opt(1, 0, 0).unwrap_or_default());
    let summer_starts = at_one(last_sunday(year, 3));
    let summer_ends = at_one(last_sunday(year, 10));
    if utc >= summer_starts && utc < summer_ends {
        2
    } else {
        1
    }
}

fn last_sunday(year: i32, month: u32) -> NaiveDate {
    let next_month = if month == 12 {
        NaiveDate::from_ymd_opt(year + 1, 1, 1)
    } else {
        NaiveDate::from_ymd_opt(year, month + 1, 1)
    }
    .unwrap_or_default();
    let mut day = next_month - Duration::days(1);
    while day.weekday() != Weekday::Sun {
        day -= Duration::days(1);
    }
    day
}

#[cfg(test)]
mod tests {
    use super::*;

    fn utc(text: &str) -> NaiveDateTime {
        NaiveDateTime::parse_from_str(text, "%Y-%m-%d %H:%M").unwrap()
    }

    #[test]
    fn winter_is_one_hour_ahead_and_summer_two() {
        assert_eq!(from_utc(utc("2026-12-01 21:30")), utc("2026-12-01 22:30"));
        assert_eq!(from_utc(utc("2026-07-01 21:30")), utc("2026-07-01 23:30"));
        // Late evening UTC is already the next day in Czechia.
        assert_eq!(
            from_utc(utc("2026-06-30 22:30")).date(),
            NaiveDate::from_ymd_opt(2026, 7, 1).unwrap()
        );
    }

    #[test]
    fn the_clocks_change_on_the_last_sundays_of_march_and_october() {
        // 2026: 29 March and 25 October.
        assert_eq!(from_utc(utc("2026-03-29 00:59")), utc("2026-03-29 01:59"));
        assert_eq!(from_utc(utc("2026-03-29 01:00")), utc("2026-03-29 03:00"));
        assert_eq!(from_utc(utc("2026-10-25 00:59")), utc("2026-10-25 02:59"));
        assert_eq!(from_utc(utc("2026-10-25 01:00")), utc("2026-10-25 02:00"));
        assert_eq!(last_sunday(2027, 3), NaiveDate::from_ymd_opt(2027, 3, 28).unwrap());
    }

    #[test]
    fn czech_midnight_in_utc() {
        assert_eq!(to_utc(utc("2026-11-01 00:00")), utc("2026-10-31 23:00"));
        assert_eq!(to_utc(utc("2026-07-01 00:00")), utc("2026-06-30 22:00"));
    }
}
