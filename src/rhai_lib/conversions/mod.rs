extern crate log;
use rhai::{Engine};

pub fn float_to_int(v: f64) -> i64 {
    v as i64
}

pub fn int_to_float(v: i64) -> f64 {
    v as f64
}

pub fn init(engine: &mut Engine) {
    log::trace!("Running STDLIB::conversions init");
    engine.register_fn("float_to_int", float_to_int);
    engine.register_fn("int_to_float", int_to_float);
}
