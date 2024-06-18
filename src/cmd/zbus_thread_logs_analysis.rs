extern crate log;
use crate::cmd;
use crate::stdlib;
use etime::Etime;
use serde_json::{json, Deserializer, Value};
use json_value_merge::Merge;


pub fn processor(c: &cmd::Cli, send_statistics: bool, sender_is_none: bool)  {
    log::debug!("zbus_thread_logs_analysis::run() reached");
    let c = c.clone();

    match stdlib::threads::THREADS.lock() {
        Ok(t) => {
            t.execute(move ||
            {
                log::debug!("LOGS ANALYSIS thread has been started");
                loop {
                    if ! stdlib::channel::pipe_is_empty_raw("logs_analysis".to_string()) {
                        match stdlib::channel::pipe_pull("logs_analysis".to_string()) {
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
                                            let data = match cmd::zbus_json::zjson_get_data(zjson.clone()) {
                                                Some(data) => data,
                                                None => continue,
                                            };
                                            if data.is_string() {
                                                let val = match data.as_str() {
                                                    Some(val) => val,
                                                    None => continue,
                                                };
                                                let analysis_data = json!({
                                                    "body": {
                                                        "details": {
                                                            "details": {
                                                                "analytical_data": {
                                                                    "category":     stdlib::logs_categorization::nbc_categorize(val.to_string()),
                                                                }
                                                            }
                                                        }
                                                    }
                                                });
                                                zjson.merge(&analysis_data);
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
                                    let data = cmd::zbus_json::generate_json_telemetry(&c, "/zbus/udg/logs_analysis/elapsed".to_string(), "Elapsed time for JSON batch processing".to_string(), 3, json!(elapsed));
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
