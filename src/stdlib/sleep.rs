use std::{thread, time};

pub fn sleep(s: i64) {
    thread::sleep(time::Duration::from_secs(s as u64));
}
pub fn sleep_millisecond(s: i64) {
    thread::sleep(time::Duration::from_millis(s as u64));
}
