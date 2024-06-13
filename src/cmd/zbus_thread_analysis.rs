extern crate log;
use crate::cmd;
use crate::stdlib;
use etime::Etime;
use serde_json::{json, Deserializer, Value};
use json_value_merge::Merge;
use statrs::statistics::Statistics;


pub fn processor(c: &cmd::Cli, send_statistics: bool, sender_is_none: bool, anomalies_window: usize)  {
    log::debug!("zbus_thread_analysis::run() reached");
    let c = c.clone();

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
                                                    Some(val) => stdlib::metric_samples::push_metric(itemkey.clone(), val),
                                                    None => log::error!("Error getting f64 metric for analysis: {}", &itemkey),
                                                }
                                                match stdlib::metric_samples::get_metric(itemkey.clone()) {
                                                    Some(mut sample) => {
                                                        let data = sample.data();
                                                        let analysis_data = json!({
                                                            "body": {
                                                                "details": {
                                                                    "details": {
                                                                        "analytical_data": {
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
                                                                            "anomalies":                sample.anomalies(anomalies_window),
                                                                            "breakouts":                sample.breakouts(anomalies_window),
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
                                if send_statistics {
                                    let data = cmd::zbus_json::generate_json_telemetry(&c, "/zbus/udg/filter/elapsed".to_string(), "Elapsed time for JSON batch processing".to_string(), 3, json!(elapsed));
                                    if ! sender_is_none {
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
