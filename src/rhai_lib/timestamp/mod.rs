extern crate log;
use std::time::{SystemTime, UNIX_EPOCH, Duration};
use rhai::{Engine};
use rhai::plugin::*;

#[derive(Debug, Clone)]
pub struct TimeInterval {
    pub s: i64,
    pub e: i64,
}

impl TimeInterval {
    pub fn new() -> Self {
        let curr = timestamp_module::timestamp_sec();
        Self {
            s: curr - 150,
            e: curr + 150,
        }
    }
    fn new_with_delta(d: i64) -> Self {
        let curr = timestamp_module::timestamp_sec();
        Self {
            s: curr - (d/2 as i64),
            e: curr + (d/2 as i64),
        }
    }
    fn new_with_delta_and_seconds(d: i64, curr: i64) -> Self {
        Self {
            s: curr - (d/2 as i64),
            e: curr + (d/2 as i64),
        }
    }
    fn new_with_delta_and_nanoseconds(d: i64, curr: f64) -> Self {
        Self {
            s: timestamp_module::whole_seconds(curr) as i64 - (d/2 as i64),
            e: timestamp_module::whole_seconds(curr) as i64 + (d/2 as i64),
        }
    }
    fn elapsed(self: &mut TimeInterval) -> i64 {
        self.e - self.s
    }
}


#[export_module]
pub mod timestamp_module {
    pub fn timestamp_sec() -> i64 {
    	SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs_f64() as i64
    }
    pub fn timestamp_ms() -> f64 {
    	SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis() as f64
    }
    pub fn timestamp_ns() -> f64 {
    	SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos() as f64
    }
    pub fn whole_seconds(t: f64) -> f64 {
        Duration::from_nanos(t as u64).as_secs_f64()
    }
    pub fn nanoseconds(t: f64) -> f64 {
        Duration::from_nanos(t as u64).subsec_nanos() as f64
    }
}

pub fn init(engine: &mut Engine) {
    log::debug!("Running STDLIB::timestamp init");

    engine.register_type::<TimeInterval>()
          .register_fn("TimeInterval",    TimeInterval::new)
          .register_fn("TimeInterval",    TimeInterval::new_with_delta)
          .register_fn("TimeInterval",    TimeInterval::new_with_delta_and_seconds)
          .register_fn("TimeInterval",    TimeInterval::new_with_delta_and_nanoseconds)
          .register_fn("elapsed",         TimeInterval::elapsed)
          .register_fn("to_string", |x: &mut TimeInterval| format!("TimeInterval({}-{}, elapsed {})", x.s, x.e, (x.e-x.s)) );

    let module = exported_module!(timestamp_module);

    engine.register_static_module("timestamp", module.into());
}
