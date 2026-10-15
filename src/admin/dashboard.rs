//! The numbers on the admin overview. Months are Czech calendar months.

use chrono::{Datelike, NaiveDate, NaiveDateTime};

use crate::czech_time;
use crate::db::Db;

#[derive(Debug, PartialEq, Eq)]
pub struct Stats {
    pub products: i64,
    pub orders: i64,
    pub orders_this_month: i64,
    pub orders_last_month: i64,
}

/// `now` is the Czech time.
pub async fn stats(db: &Db, now: NaiveDateTime) -> sqlx::Result<Stats> {
    let (this_month, last_month) = month_starts(now.date());
    let count = |sql: &'static str| sqlx::query_scalar::<_, i64>(sql);
    Ok(Stats {
        products: count("SELECT COUNT(*) FROM catalog_products").fetch_one(db).await?,
        orders: count("SELECT COUNT(*) FROM orders").fetch_one(db).await?,
        orders_this_month: count("SELECT COUNT(*) FROM orders WHERE created_at >= ?")
            .bind(&this_month)
            .fetch_one(db)
            .await?,
        orders_last_month: count("SELECT COUNT(*) FROM orders WHERE created_at >= ? AND created_at < ?")
            .bind(&last_month)
            .bind(&this_month)
            .fetch_one(db)
            .await?,
    })
}

/// The first moments (Czech midnight) of this month and of the month
/// before, in UTC as the stored times compare ("2026-09-30 22:00:00").
fn month_starts(today: NaiveDate) -> (String, String) {
    let this_month = today.with_day(1).unwrap_or(today);
    let last_month = this_month
        .pred_opt()
        .and_then(|day| day.with_day(1))
        .unwrap_or(this_month);
    let start = |day: NaiveDate| {
        czech_time::to_utc(day.and_hms_opt(0, 0, 0).unwrap_or_default())
            .format("%Y-%m-%d %H:%M:%S")
            .to_string()
    };
    (start(this_month), start(last_month))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn months_start_at_czech_midnight_and_wrap_around_the_year() {
        let day = |y, m, d| NaiveDate::from_ymd_opt(y, m, d).unwrap();
        assert_eq!(
            month_starts(day(2026, 1, 15)),
            ("2025-12-31 23:00:00".into(), "2025-11-30 23:00:00".into())
        );
        assert_eq!(
            month_starts(day(2026, 3, 31)),
            ("2026-02-28 23:00:00".into(), "2026-01-31 23:00:00".into())
        );
    }
}
