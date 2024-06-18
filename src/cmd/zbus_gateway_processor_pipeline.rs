extern crate log;
use crate::cmd;
use crate::stdlib;
use etime::Etime;
use serde_json::{json, Deserializer, Value};


pub fn processor(c: &cmd::Cli, pipeline: &cmd::Pipeline)  {
    log::trace!("zbus_gateway_processor_pipeline::run() reached");
    let c = c.clone();
    let pipeline = pipeline.clone();
    match stdlib::threads::THREADS.lock() {
        Ok(t) => {
            t.execute(move ||
            {
                log::debug!("PROCESSOR PIPELINE thread has been started");
                loop {
                    if ! stdlib::channel::pipe_is_empty_raw("in".to_string()) {
                        match stdlib::channel::pipe_pull("in".to_string()) {
                            Ok(res) => {
                                log::debug!("Received {} bytes by processor", &res.len());
                                let mut e = Etime::new();
                                e.tic();
                                let stream = Deserializer::from_str(&res).into_iter::<Value>();
                                for value in stream {
                                    match value {
                                        Ok(zjson) => {
                                            if ! zjson.is_object() {
                                                log::error!("Received JSON is not an object: {}", &zjson);
                                                continue;
                                            }
                                            let itemkey = match cmd::zbus_gateway_processor::zabbix_json_get_sub_subkey_raw(&zjson, "body".to_string(), "details".to_string(), "destination".to_string()) {
                                                Some(key) => format!("zbus/metric/{}/{}{}", &c.protocol_version, &c.platform_name, key.as_str().unwrap()),
                                                None => continue,
                                            };
                                            match &pipeline.script {
                                                Some(_) => {
                                                    stdlib::channel::pipe_push("filter".to_string(), zjson.to_string());
                                                }
                                                None => {
                                                    if pipeline.logs_analysis {
                                                        match cmd::zbus_json::zjson_get_datatype(zjson.clone()) {
                                                            Some(ct) => {
                                                                if ct == 2 {
                                                                    log::debug!("Pushing {} for logs_analysis", &itemkey);
                                                                    stdlib::channel::pipe_push("logs_analysis".to_string(), zjson.to_string());
                                                                    continue;
                                                                }
                                                            }
                                                            None => {},
                                                        }
                                                    }
                                                    if pipeline.analysis {
                                                        stdlib::channel::pipe_push("analysis".to_string(), zjson.to_string());
                                                    } else {
                                                        stdlib::channel::pipe_push("out".to_string(), zjson.to_string());
                                                    }
                                                }
                                            }
                                        }
                                        Err(err) => {
                                            log::error!("Error converting JSON: {:?}", err);
                                        }
                                    }
                                }
                                let elapsed = e.toc().as_secs_f32();
                                log::debug!("Elapsed time for processing: {} seconds", elapsed);
                                if pipeline.telemetry_monitor_elapsed {
                                    let data = cmd::zbus_json::generate_json_telemetry(&c, "/zbus/udg/elapsed".to_string(), "Elapsed time for JSON batch processing".to_string(), 3, json!(elapsed));
                                    stdlib::channel::pipe_push("out".to_string(), data.to_string());
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
