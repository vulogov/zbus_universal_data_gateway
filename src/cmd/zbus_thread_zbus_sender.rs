extern crate log;
use crate::cmd;
use crate::stdlib;
use serde_json::{Deserializer, Value};
use zenoh::prelude::sync::*;
use zenoh::config::{Config, WhatAmI};

pub fn sender(c: &cmd::Cli, config: Config, is_aggregate: bool, is_aggregate_and_split: bool, aggregate_key_topic: String)  {
    log::debug!("zbus_thread_zbus_sender::run() reached");
    let c       = c.clone();
    match stdlib::threads::THREADS.lock() {
        Ok(t) => {
            t.execute(move ||
            {
                log::debug!("ZBUS sender thread has been started");

                let aggregate_key = format!("zbus/metric/{}/{}/{}", &c.protocol_version, &c.platform_name, &aggregate_key_topic);
                if is_aggregate {
                    log::debug!("Published telemetry will be aggregated to: {}", &aggregate_key);
                } else {
                    log::debug!("Published telemetry will not be aggregated");
                }
                let receiver = match zenoh::scout(WhatAmI::Peer, config.clone())
                    .res() {
                        Ok(receiver) => receiver,
                        Err(err) => {
                            log::error!("ZBUS scout had failed for alerts: {}", err);
                            return;
                        }
                    };
                match zenoh::open(config.clone()).res() {
                    Ok(session) => {
                        loop {
                            if ! stdlib::channel::pipe_is_empty_raw("out".to_string()) {
                                match stdlib::channel::pipe_pull("out".to_string()) {
                                    Ok(res) => {
                                        log::debug!("Received {} bytes by ZBUS processor", &res.len());
                                        let stream = Deserializer::from_str(&res).into_iter::<Value>();
                                        for value in stream {
                                            match value {
                                                Ok(zjson) => {
                                                    match serde_json::to_string(&zjson) {
                                                        Ok(payload) => {
                                                            if is_aggregate {
                                                                match session.put(aggregate_key.clone(), payload.clone()).encoding(KnownEncoding::AppJson).res() {
                                                                    Ok(_) => log::debug!("ZBX catcher->ZBUS: {} len()={} bytes", &aggregate_key, &payload.len()),
                                                                    Err(err) => log::error!("Error ingesting {} {:?}: {:?}", &aggregate_key, &payload, err),
                                                                }
                                                            } else {
                                                                let itemkey = match cmd::zbus_gateway_processor::zabbix_json_get_sub_subkey_raw(&zjson, "body".to_string(), "details".to_string(), "destination".to_string()) {
                                                                    Some(key) => format!("zbus/metric/{}/{}{}", &c.protocol_version, &c.platform_name, key.as_str().unwrap()),
                                                                    None => continue,
                                                                };
                                                                match session.put(itemkey.clone(), payload.clone()).encoding(KnownEncoding::AppJson).res() {
                                                                    Ok(_) => log::debug!("ZBX catcher->ZBUS #1: {} len()={} bytes", &itemkey, &payload.len()),
                                                                    Err(err) => log::error!("Error ingesting {} {:?}: {:?}", &itemkey, &payload, err),
                                                                }
                                                                if is_aggregate_and_split {
                                                                    match session.put(aggregate_key.clone(), payload.clone()).encoding(KnownEncoding::AppJson).res() {
                                                                        Ok(_) => log::debug!("ZBX catcher->ZBUS #2: {} len()={} bytes", &aggregate_key, &payload.len()),
                                                                        Err(err) => log::error!("Error ingesting {} {:?}: {:?}", &aggregate_key, &payload, err),
                                                                    }
                                                                }
                                                            }
                                                        }
                                                        Err(err) => {
                                                            log::error!("Error convert JSON to string: {}", err);
                                                        }
                                                    }
                                                    // println!("{}", &zjson);
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
                                log::trace!("Running ZBUS scout to detect the health of connection");
                                let mut c = c.hello_received;
                                while let Ok(hello) = receiver.recv() {
                                    c -= 1;
                                    stdlib::sleep::sleep(1);
                                    if c > 0 {
                                        log::trace!("ZBUS catcher received: {}", hello);
                                        std::thread::yield_now();
                                    } else {
                                        break;
                                    }
                                }
                            }
                        }
                    }
                    Err(err) => {
                        log::error!("Error connecting to the bus: {:?}", err);
                        return;
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
