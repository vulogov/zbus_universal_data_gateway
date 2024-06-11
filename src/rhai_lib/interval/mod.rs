extern crate log;
use rhai::{Engine};
use rhai::plugin::*;

use std::ops::{Add, Sub, Mul, Div};
use inari::{Interval as I, interval};

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct Interval {
    pub i: I,
}

impl Interval {
    fn new(x: f64, y: f64) -> Self {
        Self {
            i: interval!(x, y).unwrap(),
        }
    }
    fn from_interval(i: I) -> Self {
        Self {
            i: i,
        }
    }
    fn width(self: &mut Interval) -> f64 {
        self.i.wid()
    }
    fn midpoint(self: &mut Interval) -> f64 {
        self.i.mid()
    }
    fn upper(self: &mut Interval) -> f64 {
        self.i.sup()
    }
    fn lower(self: &mut Interval) -> f64 {
        self.i.inf()
    }
    fn magnitude(self: &mut Interval) -> f64 {
        self.i.mag()
    }
    pub fn contains_in(self: &mut Interval, n: f64) -> bool {
        self.i.contains(n)
    }
    fn within(self: &mut Interval, n: f64) -> bool {
        n >= self.lower() && n <= self.upper()
    }
    fn less(self: &mut Interval, other: Interval) -> bool {
        self.i.less(other.i)
    }
    fn more(self: &mut Interval, other: Interval) -> bool {
        ! self.i.less(other.i)
    }
    fn eq(self: &mut Interval, other: Interval) -> bool {
        self.i.eq(&other.i)
    }
    fn interior(self: &mut Interval, other: Interval) -> bool {
        self.i.interior(other.i)
    }
    fn disjoint(self: &mut Interval, other: Interval) -> bool {
        self.i.disjoint(other.i)
    }
    fn interval_intersection(self: &mut Interval, other: Interval) -> Interval {
        Interval::from_interval(self.i.intersection(other.i))
    }
    fn interval_add(self: &mut Interval, other: Interval) -> Interval {
        Interval::from_interval(self.i.add(other.i))
    }
    fn interval_sub(self: &mut Interval, other: Interval) -> Interval {
        Interval::from_interval(self.i.sub(other.i))
    }
    fn interval_mul(self: &mut Interval, other: Interval) -> Interval {
        Interval::from_interval(self.i.mul(other.i))
    }
    fn interval_div(self: &mut Interval, other: Interval) -> Interval {
        Interval::from_interval(self.i.div(other.i))
    }
    fn interval_abs(self: &mut Interval) -> Interval {
        Interval::from_interval(self.i.abs())
    }
    fn interval_ceil(self: &mut Interval) -> Interval {
        Interval::from_interval(self.i.ceil())
    }
    fn interval_floor(self: &mut Interval) -> Interval {
        Interval::from_interval(self.i.floor())
    }
    fn interval_min(self: &mut Interval, other: Interval) -> Interval {
        Interval::from_interval(self.i.min(other.i))
    }
    fn interval_max(self: &mut Interval, other: Interval) -> Interval {
        Interval::from_interval(self.i.max(other.i))
    }
}

pub fn fn_make_observational_error_interval(d: f64, q: f64) -> Interval {
    let q_delta = (d*q)/2.0;
    Interval::new(d-q_delta, d+q_delta)
}

pub fn make_observational_error_interval(_context: NativeCallContext, d: f64, q: f64) -> Result<Interval, Box<EvalAltResult>> {
    Ok(fn_make_observational_error_interval(d,q))
}

#[export_module]
pub mod interval_module {
}

pub fn init(engine: &mut Engine) {
    log::trace!("Running STDLIB::interval init");
    engine.register_type::<Interval>()
          .register_fn("Interval", Interval::new)
          .register_fn("width", Interval::width)
          .register_fn("upper", Interval::upper)
          .register_fn("midpoint", Interval::midpoint)
          .register_fn("contains", Interval::contains_in)
          .register_fn("lower", Interval::lower)
          .register_fn("magnitude", Interval::magnitude)
          .register_fn("less", Interval::less)
          .register_fn("more", Interval::more)
          .register_fn("eq", Interval::eq)
          .register_fn("interior", Interval::interior)
          .register_fn("disjoint", Interval::disjoint)
          .register_fn("intersection", Interval::interval_intersection)
          .register_fn("add", Interval::interval_add)
          .register_fn("sub", Interval::interval_sub)
          .register_fn("mul", Interval::interval_mul)
          .register_fn("div", Interval::interval_div)
          .register_fn("abs", Interval::interval_abs)
          .register_fn("ceil", Interval::interval_ceil)
          .register_fn("floor", Interval::interval_floor)
          .register_fn("min", Interval::interval_min)
          .register_fn("max", Interval::interval_max)
          .register_fn("within", Interval::within)
          .register_fn("to_string", |x: &mut Interval| format!("Interval({}:{})", x.lower(), x.upper()) );

    let mut module = exported_module!(interval_module);
    module.set_native_fn("observation_error", make_observational_error_interval);
    engine.register_static_module("interval", module.into());
}
