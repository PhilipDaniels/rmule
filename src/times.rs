use rusqlite::ToSql;
use rusqlite::types::FromSql;
use time::OffsetDateTime;

pub fn now_to_yyyy_mm_dd() -> String {
    let dt = OffsetDateTime::from(std::time::SystemTime::now());
    let (y, m, d) = dt.to_calendar_date();
    format!("{}-{:0>2}-{:0>2}", y, m as u8, d)
}

/// Returns the current time as a string in the format that SQLite will understand.
pub fn now_to_sql() -> String {
    let dt = OffsetDateTime::from(std::time::SystemTime::now());
    let (y, m, d) = dt.to_calendar_date();
    let (hh, mm, ss, ms) = dt.to_hms_milli();

    format!("{}-{:0>2}-{:0>2} {:0>2}:{:0>2}:{:0>2}.{:0>3}",
        y, m as u8, d,
        hh, mm, ss, ms
        )
}

