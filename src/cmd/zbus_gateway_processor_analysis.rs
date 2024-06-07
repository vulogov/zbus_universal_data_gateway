extern crate log;
use lazy_static::lazy_static;
use crate::cmd;
use crate::stdlib;
use etime::Etime;
use std::sync::Mutex;
use serde_json::{json, Deserializer, Value};
use std::collections::btree_map::BTreeMap;
use json_value_merge::Merge;
use statrs::statistics::Statistics;

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

pub fn processor(c: &cmd::Cli, gateway: &cmd::Gateway)  {
    log::trace!("zbus_gateway_processor_analysis::run() reached");
    let c = c.clone();
    let gateway = gateway.clone();
    match stdlib::threads::THREADS.lock() {
        Ok(t) => {
            t.execute(move ||
            {
                log::debug!("ANALYSIS thread has been started");
                loop {
                    if ! stdlib::channel::pipe_is_empty_raw("analysis".to_string()) {
                        match stdlib::channel::pipe_pull("analysis".to_string()) {
                            Ok(res) => {
                                log::debug!("Received {} bytes by analysis", &res.len());
                                let mut e = Etime::new();
                                e.tic();
                                let stream = Deserializer::from_str(&res).into_iter::<Value>();
                                for value in stream {
                                    match value {
                                        Ok(mut zjson) => {
                                            if ! zjson.is_object() {
                                                log::error!("Received JSON is not an object: {}", &zjson);
                                                continue;
                                            }
                                            let itemkey = match cmd::zbus_gateway_processor::zabbix_json_get_sub_subkey_raw(&zjson, "body".to_string(), "details".to_string(), "destination".to_string()) {
                                                Some(key) => format!("zbus/metric/{}/{}{}", &c.protocol_version, &c.platform_name, key.as_str().unwrap()),
                                                None => continue,
                                            };
                                            let d = match cmd::zbus_gateway_processor::zabbix_json_get_sub_subkey_raw(&zjson, "body".to_string(), "details".to_string(), "details".to_string()) {
                                                Some(d) => d,
                                                None => continue,
                                            };
                                            let data = match cmd::zbus_gateway_processor::zabbix_json_get_raw(&d, "data".to_string()) {
                                                Some(d) => d,
                                                None => continue,
                                            };
                                            if data.is_f64() {
                                                match data.as_f64() {
                                                    Some(val) => push_metric(itemkey.clone(), val),
                                                    None => log::error!("Error getting f64 metric for analysis: {}", &itemkey),
                                                }
                                                match get_metric(itemkey.clone()) {
                                                    Some(mut sample) => {
                                                        let data = sample.data();
                                                        let analysis_data = json!({
                                                            "body": {
                                                                "details": {
                                                                    "details": {
                                                                        "analythical_data": {
                                                                            "n_of_samples":             sample.len(),
                                                                            "statistical_oscillator":   sample.oscillator(),
                                                                            "tsf_next":                 sample.tsf_next(),
                                                                            "std_dev":                  data.clone().std_dev(),
                                                                            "minimum":                  data.clone().min(),
                                                                            "maximum":                  data.clone().max(),
                                                                            "mean":                     data.clone().mean(),
                                                                            "geometric_mean":           data.clone().geometric_mean(),
                                                                            "harmonic_mean()":          data.clone().harmonic_mean(),
                                                                            "quadratic_mean":           data.clone().quadratic_mean(),
                                                                            "variance":                 data.clone().variance(),
                                                                            "markov_chain_forecast":    sample.markov(),
                                                                            "anomalies":                sample.anomalies(gateway.anomalies_window),
                                                                        }
                                                                    }
                                                                }
                                                            }
                                                        });
                                                        zjson.merge(&analysis_data);
                                                    }
                                                    None => {
                                                        log::debug!("No samples for {}", &itemkey);
                                                    }
                                                }
                                            }
                                            stdlib::channel::pipe_push("out".to_string(), zjson.to_string());
                                        }
                                        Err(err) => {
                                            log::error!("Error converting JSON: {:?}", err);
                                        }
                                    }
                                }
                                let elapsed = e.toc().as_secs_f32();
                                log::debug!("Elapsed time for analysis: {} seconds", elapsed);
                                if gateway.telemetry_monitor_elapsed {
                                    let data = cmd::zbus_json::generate_json_telemetry(&c, "/zbus/udg/filter/elapsed".to_string(), "Elapsed time for JSON batch processing".to_string(), 3, json!(elapsed));
                                    if ! gateway.group.none {
                                        stdlib::channel::pipe_push("out".to_string(), data.to_string());
                                    }
                                }
                            }
                            Err(err) => log::error!("Error getting data from channel: {:?}", err),
                        }
                    } else {
                        stdlib::sleep::sleep(1);
                    }
                }
            });
            drop(t);
        }
        Err(err) => {
            log::error!("Error accessing Thread Manager: {:?}", err);
            return;
        }
    }
}
