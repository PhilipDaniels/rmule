use time::OffsetDateTime;

pub fn current_date_to_yyyy_mm_dd() -> String {
    let dt = OffsetDateTime::from(std::time::SystemTime::now());
    format!("{}-{:0>2}-{:0>2}", dt.year(), dt.month() as u8, dt.day())
}

