//! The clock PyBot reads: `datetime.date.today()` and `datetime.datetime.now(timezone.utc)`.
//!
//! One place for every module. Real time by default; pinned to a day with `RSBOT_TODAY=YYYY-MM-DD`
//! (noon UTC), which is how every oracle was recorded (`--today 2026-11-17`), so checkers set it.
//! Dates are `chrono::NaiveDate`; Python's `date.weekday()` is `.weekday().num_days_from_monday()`,
//! `date - timedelta(days=n)` is `d - chrono::Days::new(n)`.

use chrono::{DateTime, NaiveDate, TimeZone, Utc};
use std::sync::OnceLock;

fn pinned() -> Option<NaiveDate> {
    static PIN: OnceLock<Option<NaiveDate>> = OnceLock::new();
    *PIN.get_or_init(|| std::env::var("RSBOT_TODAY").ok().and_then(|s| NaiveDate::parse_from_str(&s, "%Y-%m-%d").ok()))
}

/// `datetime.date.today()` (local date).
pub fn today() -> NaiveDate {
    pinned().unwrap_or_else(local_today)
}

#[cfg(not(target_family = "wasm"))]
fn local_today() -> NaiveDate {
    chrono::Local::now().date_naive()
}

/// The local time zone's offset from UTC, in minutes, as the WebAssembly host reports it.
#[cfg(target_family = "wasm")]
static LOCAL_OFFSET_MINUTES: std::sync::atomic::AtomicI32 = std::sync::atomic::AtomicI32::new(0);

/// In WebAssembly chrono cannot see the time zone (its `Local` is UTC there), so the page
/// passes the browser's offset (east of UTC, in minutes) and the local date is computed from it.
#[cfg(target_family = "wasm")]
pub fn set_local_offset_minutes(minutes: i32) {
    LOCAL_OFFSET_MINUTES.store(minutes, std::sync::atomic::Ordering::Relaxed);
}

#[cfg(target_family = "wasm")]
fn local_today() -> NaiveDate {
    let minutes = LOCAL_OFFSET_MINUTES.load(std::sync::atomic::Ordering::Relaxed);
    (Utc::now() + chrono::Duration::minutes(minutes as i64)).date_naive()
}

/// `datetime.datetime.now(datetime.timezone.utc)`.
pub fn now_utc() -> DateTime<Utc> {
    match pinned() {
        Some(d) => Utc.from_utc_datetime(&d.and_hms_opt(12, 0, 0).unwrap()),
        None => Utc::now(),
    }
}

