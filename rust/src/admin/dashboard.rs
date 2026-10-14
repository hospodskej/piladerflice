//! The numbers on the admin overview. Months are calendar months in UTC,
//! as in the Rails app (it has no time zone set).

use chrono::{Datelike, NaiveDate, NaiveDateTime};

use crate::db::Db;

#[derive(Debug, PartialEq, Eq)]
pub struct Stats {
    pub products: i64,
    pub orders: i64,
    pub orders_this_month: i64,
    pub orders_last_month: i64,
}

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

/// The first moments of this month and of the month before, as stored
/// timestamps compare ("2026-10-01 00:00:00").
fn month_starts(today: NaiveDate) -> (String, String) {
    let this_month = today.with_day(1).unwrap_or(today);
    let last_month = this_month
        .pred_opt()
        .and_then(|day| day.with_day(1))
        .unwrap_or(this_month);
    let start = |day: NaiveDate| day.format("%Y-%m-%d 00:00:00").to_string();
    (start(this_month), start(last_month))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn last_month_wraps_around_the_year() {
        let day = |y, m, d| NaiveDate::from_ymd_opt(y, m, d).unwrap();
        assert_eq!(
            month_starts(day(2026, 1, 15)),
            ("2026-01-01 00:00:00".into(), "2025-12-01 00:00:00".into())
        );
        assert_eq!(
            month_starts(day(2026, 3, 31)),
            ("2026-03-01 00:00:00".into(), "2026-02-01 00:00:00".into())
        );
    }
}
