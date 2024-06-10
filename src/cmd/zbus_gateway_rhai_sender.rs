extern crate log;
use crate::cmd;
use crate::stdlib;
use serde_json::{Deserializer, Value, from_str};
use rhai::{Dynamic, Map};


pub fn sender(c: &cmd::Cli, gateway: &cmd::Gateway)  {
    log::trace!("zbus_gateway_rhai_sender::run() reached");
    let gateway = gateway.clone();
    let c = c.clone();
    match stdlib::threads::THREADS.lock() {
        Ok(t) => {
            t.execute(move ||
            {
                log::debug!("RHAI sender thread has been started");
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

                    if ! stdlib::channel::pipe_is_empty_raw("out".to_string()) {
                        match stdlib::channel::pipe_pull("out".to_string()) {
                            Ok(res) => {
                                log::debug!("Received {} bytes by RHAI processor", &res.len());
                                let vstream = Deserializer::from_str(&res).into_iter::<Value>();
                                for value in vstream {
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
                                                        match engine.call_fn::<Map>(&mut scope, &ast, "processor", (val,)) {
                                                            Ok(_) => {

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
                                    log::debug!("End of JSON");
                                }
                                log::debug!("End of JSON series.");
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
