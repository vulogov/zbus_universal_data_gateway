extern crate log;
use rhai::{Engine, EvalAltResult, Dynamic, Array};
use rhai::plugin::*;
use std::collections::VecDeque;

use crate::rhai_lib::{timestamp, metric, string, interval};

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct DynamicSampler {
    pub k:          String,
    pub s:          i64,
    d:              VecDeque<metric::Metric>,
}

impl DynamicSampler {
    fn new() -> Self {
        Self {
            k: "".to_string(),
            s: 128 as i64,
            d: VecDeque::new(),
        }
    }
    fn data(self: &mut DynamicSampler) -> Result<Array, Box<EvalAltResult>> {
        let mut res = Array::new();
        for m in self.d.iter() {
            match m.clone().get_value() {
                Ok(data) => {
                    res.push(data.clone());
                }
                Err(err) => return Err(format!("{:?}", err).into()),
            }
        }
        Ok(res)
    }
    fn values(self: &mut DynamicSampler) -> Result<Array, Box<EvalAltResult>> {
        let mut res = Array::new();
        for m in self.d.iter() {
            match m.clone().get_value() {
                Ok(data) => {
                    let mut row = Array::new();
                    row.push(Dynamic::from(m.timestamp));
                    row.push(data.clone());
                    res.push(row.into());
                }
                Err(err) => return Err(format!("{:?}", err).into()),
            }
        }
        Ok(res)
    }
    fn len(self: &mut DynamicSampler) -> i64 {
        self.d.len() as i64
    }
    fn set(self: &mut DynamicSampler, m: metric::Metric) -> bool {
        let key = m.key.clone();
        if self.k.len() == 0 {
            self.k = key.clone();
        }
        if self.k != key {
            log::warn!("DynamicSampler().set() key mismatch {} <> {}", &self.k, &key);
            return false;
        }
        if self.is_timestamp(m.clone()) {
            log::warn!("DynamicSampler().set() timestamp is not unique");
            return false;
        }
        self.d.push_back(m.clone());
        if self.d.len() as i64 > self.s {
            let _ = self.d.pop_front();
        }
        true
    }
    fn is_timestamp(self: &mut DynamicSampler, m: metric::Metric) -> bool {
        for n in self.d.iter() {
            if n.timestamp == m.timestamp {
                return true;
            }
        }
        false
    }
    fn timestamp_interval(self: &mut DynamicSampler) -> Result<timestamp::TimeInterval, Box<EvalAltResult>> {
        let mut s: i64 = i64::MAX;
        let mut e: i64 = 0;
        if self.len() <= 2 {
            return Err("DynamicSampler() do not have enough data for DynamicSampler().timestamp_interval()".into());
        }
        for n in self.d.iter() {
            let t: i64 = timestamp::timestamp_module::whole_seconds(n.timestamp) as i64;
            if t < s {
                s = t;
            }
            if t > e {
                e = t
            }
        }
        if s == e {
            return Err("DynamicSampler() can not have start and end of the interval same for DynamicSampler().timestamp_interval()".into());
        }
        let mut res = timestamp::TimeInterval::new();
        res.s = s;
        res.e = e;
        return Ok(res);
    }
    fn set_unique_q_int(self: &mut DynamicSampler, q: i64, m: metric::Metric) -> Result<bool, Box<EvalAltResult>> {
        self.set_unique_q_float(q as f64, m.clone())
    }
    fn set_unique_q_float(self: &mut DynamicSampler, q: f64, m: metric::Metric) -> Result<bool, Box<EvalAltResult>> {
        let value = match m.clone().get_value() {
            Ok(data) => data,
            Err(_) => {
                return Err(format!("Can not get value for DynamicSampler().set_unique()").into());
            }
        };
        if value.is_string() {
            let s_value = value.clone().into_string();
            let mut curr_q: f64 = 0.0;
            for n in self.d.iter() {
                let value2 = match n.clone().get_value() {
                    Ok(data) => data,
                    Err(_) => {
                        return Err(format!("Can not get value for DynamicSampler().set_unique()").into());
                    }
                };
                let c = string::fuzzy::str_match_levenshtein_raw(s_value.clone().unwrap(), value2.into_string().unwrap()) as f64;
                if c > curr_q {
                    curr_q = c;
                }
            }
            if curr_q < q {
                return Ok(self.set(m.clone()));
            }
        } if value.is_float() {
            let v: f64 = value.as_float().unwrap();
            for n in self.d.iter() {
                let n_value = match n.clone().get_value() {
                    Ok(data) => data,
                    Err(_) => {
                        return Err(format!("Can not get value for DynamicSampler().set_unique()").into());
                    }
                };
                if n_value.is_float() {
                    let mut i = interval::fn_make_observational_error_interval(n_value.as_float().unwrap(), q);
                    if i.contains_in(v) {
                        return Ok(false);
                    }
                }
            }
            return Ok(self.set(m.clone()));
        } else {
            return Err(format!("Datatype stored in Metric() is not suitable for DynamicSampler().set_unique()").into());
        }
    }
}



#[export_module]
pub mod dynamicsampler_module {
}

pub fn init(engine: &mut Engine) {
    log::trace!("Running STDLIB::interval init");
    engine.register_type::<DynamicSampler>()
          .register_fn("Sampler", DynamicSampler::new)
          .register_fn("data", DynamicSampler::data)
          .register_fn("values", DynamicSampler::values)
          .register_fn("len", DynamicSampler::len)
          .register_fn("set", DynamicSampler::set)
          .register_fn("set_unique", DynamicSampler::set_unique_q_int)
          .register_fn("set_unique", DynamicSampler::set_unique_q_float)
          .register_fn("is_timestamp", DynamicSampler::is_timestamp)
          .register_fn("timestamp_interval", DynamicSampler::timestamp_interval)
          .register_fn("to_string", |x: &mut DynamicSampler| format!("Sampler().len() = {}", x.len()));

    let module = exported_module!(dynamicsampler_module);
    engine.register_static_module("sampler", module.into());
}
