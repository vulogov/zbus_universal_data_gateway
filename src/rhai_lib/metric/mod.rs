extern crate log;
use rhai::{Engine, Dynamic, EvalAltResult, Map, Identifier};
use rhai::plugin::*;
use serde_json::json;
use rhai::serde::{to_dynamic};

use crate::rhai_lib::timestamp;
use crate::stdlib::hostname;

#[derive(Debug, Clone)]
pub struct Metric {
    pub key:        String,
    value:          Dynamic,
    pub timestamp:  f64,
    pub tags:       Map,
    pub platform:   String,
    pub skey:       String,
    pub src:        String,
}

impl Metric {
    fn new(key: String) -> Self {
        Self {
            key:        key.clone(),
            value:      Dynamic::UNIT,
            timestamp:  timestamp::timestamp_module::timestamp_ns(),
            tags:       Map::new(),
            platform:   "local".to_string(),
            skey:       (*key.clone().split("/").collect::<Vec<_>>().last().unwrap()).to_string(),
            src:        hostname::get_hostname(),
        }
    }
    fn set_value(self: &mut Metric, value: Dynamic) -> Self {
        self.value = value.clone();
        return self.clone();
    }
    fn set_current_timestamp(self: &mut Metric) -> Self {
        self.timestamp = timestamp::timestamp_module::timestamp_ns();
        return self.clone();
    }
    fn set_timestamp(self: &mut Metric, t: f64) -> Self {
        self.timestamp = t;
        return self.clone();
    }
    pub fn get_value(self: &mut Metric) -> Result<Dynamic, Box<EvalAltResult>> {
        if self.value.is_unit() {
            return Err(format!("Metric() value is not initialized").into());
        }
        Ok(self.value.clone())
    }
    fn set_tag(self: &mut Metric, key: String, value: Dynamic) {
        self.tags.insert(Identifier::from(key), value.clone());
    }
    fn set_tag_fun(self: &mut Metric, key: String, value: Dynamic) -> Self {
        self.set_tag(key, value);
        return self.clone()
    }
    fn to_json(self: &mut Metric) -> Result<Dynamic, Box<EvalAltResult>> {
        match to_dynamic(json!({
            "key":          self.key,
            "value":        self.value,
            "timestamp":    self.timestamp,
            "tags":         self.tags,
            "platform":     self.platform,
            "skey":         self.skey,
            "src":          self.src,
        })) {
            Ok(res) => Ok(res),
            Err(err) => Err(format!("Error converting Metric(): {:?}", err).into())
        }
    }
    fn as_string(self: &mut Metric) -> Result<Dynamic, Box<EvalAltResult>> {
        match self.to_json() {
            Ok(jself) => {
                match serde_json::to_string(&jself) {
                    Ok(res) => Ok(res.into()),
                    Err(err) => Err(format!("{:?}", err).into()),
                }
            }
            Err(err) => {
                return Err(format!("{:?}", err).into());
            }
        }
    }
}

pub fn metric_from_string_raw(s: String) -> Result<Metric, Box<EvalAltResult>> {
    match serde_json::from_str::<Map>(&s) {
        Ok(value) => {
            return metric_from_json_raw(value);
        }
        Err(err) => {
            return Err(format!("Error converting Metric from JSON: {:?}", err).into());
        }
    }
}


pub fn metric_from_json_raw(j: Map) -> Result<Metric, Box<EvalAltResult>> {
    match j.get("key") {
        Some(key) => {
            let mut m: Metric = Metric::new(key.to_string());
            let mut m = match j.get("value") {
                Some(value) => m.set_value(value.clone()),
                None => m
            };
            let mut m = match j.get("timestamp") {
                Some(timestamp) => m.set_timestamp(timestamp.as_float().unwrap()),
                None => m
            };
            match j.get("platform") {
                Some(platform) => m.platform = platform.to_string().clone(),
                None => {}
            };
            match j.get("skey") {
                Some(skey) => m.skey = skey.to_string().clone(),
                None => {}
            };
            match j.get("src") {
                Some(src) => m.src = src.to_string().clone(),
                None => {}
            };
            match j.get("tags") {
                Some(tags) => m.tags = tags.clone_cast::<Map>(),
                None => {}
            };
            return Ok(m);
        }
        None => {
            return Err(format!("Can not make Metric from JSON argument key is missed").into())
        }
    }
}

pub fn metric_from_json(_context: NativeCallContext, j: Map) -> Result<Metric, Box<EvalAltResult>> {
    metric_from_json_raw(j)
}

pub fn metric_from_string(_context: NativeCallContext, s: String) -> Result<Metric, Box<EvalAltResult>> {
    metric_from_string_raw(s)
}

#[export_module]
pub mod metric_module {
}

pub fn init(engine: &mut Engine) {
    log::trace!("Running STDLIB::metric init");
    engine.register_type::<Metric>()
          .register_fn("Metric",    Metric::new)
          .register_fn("value",     Metric::set_value)
          .register_fn("value",     Metric::get_value)
          .register_fn("timestamp", Metric::set_current_timestamp)
          .register_fn("timestamp", Metric::set_timestamp)
          .register_fn("tag",       Metric::set_tag_fun)
          .register_fn("json",      Metric::to_json)
          .register_fn("as_string", Metric::as_string)
          .register_indexer_set(Metric::set_tag)
          .register_fn("to_string", |x: &mut Metric| format!("Metric({})", x.key) );

    let mut module = exported_module!(metric_module);
    module.set_native_fn("convert_from_json", metric_from_json);
    module.set_native_fn("convert_from_string", metric_from_string);
    engine.register_static_module("metric", module.into());
}
