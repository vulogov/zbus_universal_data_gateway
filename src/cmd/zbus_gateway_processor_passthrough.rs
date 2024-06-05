extern crate log;
use crate::cmd;
use crate::stdlib;
use etime::Etime;
use serde_json::{json, Deserializer, Value};


pub fn processor(c: &cmd::Cli, gateway: &cmd::Gateway)  {
    log::trace!("zbus_gateway_processor_passthrough::run() reached");
    let c = c.clone();
    let gateway = gateway.clone();
    match stdlib::threads::THREADS.lock() {
        Ok(t) => {
            t.execute(move ||
            {
                log::debug!("PROCESSOR PASSTHROUGH thread has been started");
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
                                            stdlib::channel::pipe_push("out".to_string(), zjson.to_string());
                                        }
                                        Err(err) => {
                                            log::error!("Error converting JSON: {:?}", err);
                                        }
                                    }
                                }
                                let elapsed = e.toc().as_secs_f32();
                                log::debug!("Elapsed time for processing: {} seconds", elapsed);
                                if gateway.telemetry_monitor_elapsed {
                                    let data = cmd::zbus_json::generate_json_telemetry(&c, "/zbus/udg/elapsed".to_string(), "Elapsed time for JSON batch processing".to_string(), 3, json!(elapsed));
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
