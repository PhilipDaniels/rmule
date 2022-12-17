use time::OffsetDateTime;

pub fn now() -> OffsetDateTime {
    OffsetDateTime::now_utc()
}

pub fn now_to_yyyy_mm_dd() -> String {
    let dt = now();
    let (y, m, d) = dt.to_calendar_date();
    format!("{}-{:0>2}-{:0>2}", y, m as u8, d)
}

pub fn now_to_yyyy_mm_dd_hh_mm_ss() -> String {
    let dt = now();
    let (y, m, d) = dt.to_calendar_date();
    let (hh, mm, ss, _ms) = dt.to_hms_milli();
    format!(
        "{}-{:0>2}-{:0>2}T{:0>2}-{:0>2}-{:0>2}",
        y, m as u8, d, hh, mm, ss
    )
}
