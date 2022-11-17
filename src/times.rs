use std::fs::read;
use std::io::Write;
use std::sync::{Arc, RwLock};
use std::thread;
use std::time::Duration;

use once_cell::sync::Lazy;
use time::OffsetDateTime;

struct CachedTimeData {
    year: i32,
    month: u8,
    day: u8,
    minute: u8,
    ymd: String
}

impl CachedTimeData {
    fn new() -> Self {
        let dt = Self::current_time();

        Self {
            year: dt.year(),
            month: dt.month() as u8,
            day: dt.day(),
            minute: dt.minute(),
            ymd: format!("{}-{:0>2}-{:0>2}", dt.year(), dt.month() as u8, dt.day())
        }
    }

    fn current_time() -> OffsetDateTime {
        std::time::SystemTime::now().into()
    }
}

pub fn create_background_date_thread() {
    thread::spawn(|| {
        loop {
            println!("Looping");

            let read_guard = CACHED_TIME.read().unwrap();
            let (day, min) = (read_guard.day, read_guard.minute);
            drop(read_guard);

            let current = CachedTimeData::current_time();

            if current.day() != day || current.minute() != min {
                println!("Updating cached time");
                let mut guard = CACHED_TIME.write().unwrap();
                *guard = CachedTimeData::new();
                println!("Updated");
            }

            thread::sleep(Duration::from_secs(10));
        }
    });
}

static CACHED_TIME: Lazy<RwLock<CachedTimeData>> = Lazy::new(|| RwLock::new(CachedTimeData::new()));


pub fn current_date_to_yyyy_mm_dd() -> String {
    let st = std::time::SystemTime::now();
    let dt: OffsetDateTime = st.into();
    format!("{}-{:0>2}-{:0>2}", dt.year(), dt.month() as u8, dt.day())
    //systemtime_strftime(st, "%Y-%m-%d")
}

// fn systemtime_strftime<T>(dt: T, format: &str) -> String
//    where T: Into<OffsetDateTime>
// {
//     let dt = dt.into();
//     format!("{}-{}-{}", dt.year(), dt.month(), dt.day())
// }
