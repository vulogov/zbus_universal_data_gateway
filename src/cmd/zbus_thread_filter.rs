extern crate log;
use crate::cmd;
use crate::stdlib;
use etime::Etime;
use serde_json::{json, Deserializer, Value};
use serde_json::{from_str};
use rhai::{Dynamic, Map};

pub fn processor(c: &cmd::Cli, script: String, send_statistics: bool, sender_is_none: bool)  {
    log::trace!("zbus_thread_filter::run() reached");
    let c = c.clone();

    match stdlib::threads::THREADS.lock() {
        Ok(t) => {
            t.execute(move ||
            {
                log::debug!("PROCESSOR FILTER thread has been started");

                let (engine, mut scope, ast) = match cmd::zbus_rhai::make_rhai_env_and_ast(script.clone(), &c) {
                    Ok((engine, scope, ast)) => (engine, scope, ast),
                    Err(err) => {
                        log::error!("Error creating RHAI instance: {}", err);
                        return;
                    }
                };

                loop {
                    if ! stdlib::channel::pipe_is_empty_raw("filter".to_string()) {
                        match stdlib::channel::pipe_pull("filter".to_string()) {
                            Ok(res) => {
                                log::debug!("Received {} bytes by filter", &res.len());
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
                                            match from_str::<Dynamic>(&zjson.to_string()) {
                                                Ok(res) => {
                                                    if res.is_map() {
                                                        let val = res.clone_cast::<Map>();
                                                        match engine.call_fn::<bool>(&mut scope, &ast, "filter", (val,)) {
                                                            Ok(filter_result) => {
                                                                if filter_result {
                                                                    stdlib::channel::pipe_push("transformation".to_string(), zjson.to_string());
                                                                } else {
                                                                    log::debug!("FILER blocked out: {}", &zjson.to_string());
                                                                }
                                                            }
                                                            Err(err) => {
                                                                log::error!("Error in filter: {}", err);
                                                            }
                                                        }
                                                    } else {
                                                        log::error!("Value is not of Map type");
                                                    }
                                                }
                                                Err(err) => {
                                                    log::error!("Error converting from JSON: {}", err);
                                                }
                                            }
                                        }
                                        Err(err) => {
                                            log::error!("Error converting JSON: {:?}", err);
                                        }
                                    }
                                }
                                let elapsed = e.toc().as_secs_f32();
                                log::debug!("Elapsed time for filtering: {} seconds", elapsed);
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
