use std::time::{SystemTime, UNIX_EPOCH, Duration};

pub fn timestamp_ns() -> f64 {
    SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos() as f64
}
pub fn whole_seconds(t: f64) -> f64 {
    Duration::from_nanos(t as u64).as_secs_f64()
}
pub fn nanoseconds(t: f64) -> f64 {
    Duration::from_nanos(t as u64).subsec_nanos() as f64
}

pub fn make_nanosecond_ts(s: f64, ns: f64) -> f64 {
    (s * 1000000000.0) + ns
}
