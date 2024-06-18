extern crate log;
use lazy_static::lazy_static;
use crate::cmd;
use std::sync::Mutex;
use std::collections::btree_map::BTreeMap;

lazy_static! {
    static ref SAMPLES: Mutex<BTreeMap<String, cmd::zbus_sampler::Sampler>> = {
        let m: Mutex<BTreeMap<String, cmd::zbus_sampler::Sampler>> = Mutex::new(BTreeMap::new());
        m
    };
}

pub fn create_pipe(n: String) {
    log::debug!("Create metric for analysis: {}", &n);
    let mut q = SAMPLES.lock().unwrap();
    q.insert(n.to_string(), cmd::zbus_sampler::Sampler::init());
    drop(q);
}

pub fn push_metric(k: String, v: f64) {
    let mut s = SAMPLES.lock().unwrap();
    if ! s.contains_key(&k) {
        let mut sample = cmd::zbus_sampler::Sampler::init();
        sample.set(v);
        s.insert(k, sample);
    } else {
        let sample = s.get_mut(&k).unwrap();
        sample.set(v);
    }
    drop(s);
}

pub fn get_metric(k: String) -> Option<cmd::zbus_sampler::Sampler> {
    let s = SAMPLES.lock().unwrap();
    if ! s.contains_key(&k) {
        drop(s);
        return None;
    }
    let sample = s.get(&k).unwrap().clone();
    drop(s);
    return Some(sample);
}
