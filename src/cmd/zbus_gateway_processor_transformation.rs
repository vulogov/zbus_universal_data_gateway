extern crate log;
use crate::cmd;
use crate::stdlib;
use etime::Etime;
use serde_json::{json, Deserializer, Value, from_str};
use rhai::{Dynamic,Map};


pub fn processor(c: &cmd::Cli, gateway: &cmd::Gateway)  {
    log::trace!("zbus_gateway_processor_filter::run() reached");
    let c = c.clone();
    let gateway = gateway.clone();
    match stdlib::threads::THREADS.lock() {
        Ok(t) => {
            t.execute(move ||
            {
                log::debug!("PROCESSOR TRANSFORMATION thread has been started");
                let script = match stdlib::zio::read_file(gateway.script.clone().unwrap()) {
                    Some(script) => script,
                    None => {
                        log::error!("Can not get the RHAI script");
                        return;
                    }
                };
                let (engine, mut scope, ast) = match cmd::zbus_rhai::make_rhai_env_and_ast(script, &c) {
                    Ok((engine, scope, ast)) => (engine, scope, ast),
                    Err(err) => {
                        log::error!("Error creating RHAI instance: {}", err);
                        return;
                    }
                };

                loop {
                    if ! stdlib::channel::pipe_is_empty_raw("transformation".to_string()) {
                        match stdlib::channel::pipe_pull("transformation".to_string()) {
                            Ok(res) => {
                                log::debug!("Received {} bytes by transformation", &res.len());
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
                                                        match engine.call_fn::<Map>(&mut scope, &ast, "transformation", (val,)) {
                                                            Ok(transform_result) => {
                                                                match serde_json::to_string(&transform_result) {
                                                                    Ok(transform_result_str) => {
                                                                        stdlib::channel::pipe_push("out".to_string(), transform_result_str);
                                                                    }
                                                                    Err(err) => {
                                                                        log::error!("Error converting transformation to JSON: {}", err);
                                                                    }
                                                                }
                                                            }
                                                            Err(err) => {
                                                                log::error!("Error in transformation processor: {}", err);
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
                                if gateway.telemetry_monitor_elapsed {
                                    let data = cmd::zbus_json::generate_json_telemetry(&c, "/zbus/udg/transformation/elapsed".to_string(), "Elapsed time for JSON batch processing".to_string(), 3, json!(elapsed));
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
