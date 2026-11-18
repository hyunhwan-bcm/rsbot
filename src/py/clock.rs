//! The clock PyBot reads: `datetime.date.today()` and `datetime.datetime.now(timezone.utc)`.
//!
//! One place for every module. Real time by default; pinned to a day with `RSBOT_TODAY=YYYY-MM-DD`
//! (noon UTC), which is how every oracle was recorded (`--today 2026-11-17`), so checkers set it.
//! Dates are `chrono::NaiveDate`; Python's `date.weekday()` is `.weekday().num_days_from_monday()`,
//! `date - timedelta(days=n)` is `d - chrono::Days::new(n)`.

use chrono::{DateTime, Local, NaiveDate, TimeZone, Utc};
use std::sync::OnceLock;

fn pinned() -> Option<NaiveDate> {
    static PIN: OnceLock<Option<NaiveDate>> = OnceLock::new();
    *PIN.get_or_init(|| std::env::var("RSBOT_TODAY").ok().and_then(|s| NaiveDate::parse_from_str(&s, "%Y-%m-%d").ok()))
}

/// `datetime.date.today()` (local date).
pub fn today() -> NaiveDate {
    pinned().unwrap_or_else(|| Local::now().date_naive())
}

/// `datetime.datetime.now(datetime.timezone.utc)`.
pub fn now_utc() -> DateTime<Utc> {
    match pinned() {
        Some(d) => Utc.from_utc_datetime(&d.and_hms_opt(12, 0, 0).unwrap()),
        None => Utc::now(),
    }
}

