//! Dates and times as the site shows them, in Czech local time.

#[cfg(feature = "ssr")]
use time::{OffsetDateTime, UtcOffset};

/// Prague's offset from UTC at `at` (CET, or CEST from the last Sunday of
/// March to the last Sunday of October, at 01:00 UTC).
#[cfg(feature = "ssr")]
pub fn prague_offset(at: OffsetDateTime) -> UtcOffset {
    use time::{Date, Month, Time, Weekday};
    let last_sunday = |month: Month| {
        let mut day = Date::from_calendar_date(at.year(), month, 31).unwrap_or(at.date());
        while day.weekday() != Weekday::Sunday {
            day = day.previous_day().unwrap_or(day);
        }
        day.with_time(Time::from_hms(1, 0, 0).unwrap_or(Time::MIDNIGHT)).assume_utc()
    };
    let summer = at >= last_sunday(Month::March) && at < last_sunday(Month::October);
    UtcOffset::from_hms(if summer { 2 } else { 1 }, 0, 0).unwrap_or(UtcOffset::UTC)
}

/// "13.11.2026 21:05".
#[cfg(feature = "ssr")]
pub fn format_cz(at: OffsetDateTime) -> String {
    let local = at.to_offset(prague_offset(at));
    format!("{:02}.{:02}.{} {:02}:{:02}", local.day(), local.month() as u8, local.year(), local.hour(), local.minute())
}

#[cfg(feature = "ssr")]
pub fn now_cz() -> String {
    format_cz(OffsetDateTime::now_utc())
}

#[cfg(all(test, feature = "ssr"))]
mod tests {
    use super::*;
    use time::macros::datetime;

    #[test]
    fn summer_and_winter_time() {
        assert_eq!(format_cz(datetime!(2026-11-13 20:05 UTC)), "13.11.2026 21:05");
        assert_eq!(format_cz(datetime!(2026-07-01 10:00 UTC)), "01.07.2026 12:00");
        // the switch: 29 March 2026 01:00 UTC
        assert_eq!(format_cz(datetime!(2026-03-29 00:59 UTC)), "29.03.2026 01:59");
        assert_eq!(format_cz(datetime!(2026-03-29 01:00 UTC)), "29.03.2026 03:00");
    }
}
