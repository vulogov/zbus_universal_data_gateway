extern crate log;
use crate::cmd;
use crate::stdlib;
use serde_json::{Deserializer, Value};

pub fn sender(_c: &cmd::Cli, gateway: &cmd::Gateway)  {
    log::trace!("zbus_gateway_stdout_sender::run() reached");
    let gateway = gateway.clone();
    if gateway.pretty {
        log::debug!("Pretty STDOUT output requested");
    } else {
        log::debug!("Plain STDOUT output requested");
    }
    match stdlib::threads::THREADS.lock() {
        Ok(t) => {
            t.execute(move ||
            {
                log::debug!("STDOUT sender thread has been started");
                loop {
                    if ! stdlib::channel::pipe_is_empty_raw("out".to_string()) {
                        match stdlib::channel::pipe_pull("out".to_string()) {
                            Ok(res) => {
                                log::debug!("Received {} bytes by STDOUT processor", &res.len());
                                let stream = Deserializer::from_str(&res).into_iter::<Value>();
                                for value in stream {
                                    match value {
                                        Ok(zjson) => {
                                            if gateway.pretty {
                                                match serde_json::to_string_pretty(&zjson) {
                                                    Ok(val) => {
                                                        println!("{}", &val);
                                                    }
                                                    Err(err) => {
                                                        log::error!("Error converting JSON for stdout: {:?}", err);
                                                    }
                                                }
                                            } else {
                                                println!("{}", &zjson);
                                            }
                                        }
                                        Err(err) => {
                                            log::error!("Error converting JSON: {:?}", err);
                                        }
                                    }
                                    log::debug!("End of JSON");
                                }
                                log::debug!("End of JSON series");
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
