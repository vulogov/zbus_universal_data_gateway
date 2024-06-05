extern crate log;
use crate::cmd;
use crate::stdlib;
use std::str::FromStr;
use serde_json::{Deserializer, Value};
use zenoh::prelude::sync::*;
use zenoh::config::{Config, ConnectConfig, ListenConfig, EndPoint, WhatAmI};

pub fn sender(c: &cmd::Cli, gateway: &cmd::Gateway)  {
    log::trace!("zbus_gateway_zbus_sender::run() reached");
    let gateway = gateway.clone();
    let c       = c.clone();
    match stdlib::threads::THREADS.lock() {
        Ok(t) => {
            t.execute(move ||
            {
                log::debug!("ZBUS sender thread has been started");
                let mut config =  Config::default();

                if gateway.zbus_disable_multicast_scout.clone() {
                    match config.scouting.multicast.set_enabled(Some(false)) {
                        Ok(_) => { log::debug!("Multicast discovery disabled")}
                        Err(err) => {
                            log::error!("Failure in disabling multicast discovery: {:?}", err);
                            return;
                        }
                    }
                } else {
                    log::debug!("Multicast discovery enabled");
                }
                match EndPoint::from_str(&gateway.zbus_connect) {
                    Ok(zconn) => {
                        log::debug!("ZENOH bus set to: {:?}", &zconn);
                        let _ = config.set_connect(ConnectConfig::new(vec![zconn]).unwrap());
                    }
                    Err(err) => {
                        log::error!("Failure in parsing connect address: {:?}", err);
                        return;
                    }
                }
                match EndPoint::from_str(&gateway.zbus_listen) {
                    Ok(zlisten) => {
                        log::debug!("ZENOH listen set to: {:?}", &zlisten);
                        let _ = config.set_listen(ListenConfig::new(vec![zlisten]).unwrap());
                    }
                    Err(_) => {
                        log::debug!("ZENOH listen set to default");
                    }
                }
                if gateway.zbus_set_connect_mode {
                    log::debug!("ZENOH configured in CONNECT mode");
                    let _ = config.set_mode(Some(WhatAmI::Client));
                } else {
                    log::debug!("ZENOH configured in PEER mode");
                    let _ = config.set_mode(Some(WhatAmI::Peer));
                }
                if config.validate() {
                    log::debug!("ZENOH config is OK");
                } else {
                    log::error!("ZENOH config not OK");
                    return;
                }
                let aggregate_key = format!("zbus/metric/{}/{}/{}", &c.protocol_version, &c.platform_name, &gateway.zbus_aggregate_key);
                if gateway.zbus_aggregate {
                    log::debug!("Published telemetry will be aggregated to: {}", &aggregate_key);
                } else {
                    log::debug!("Published telemetry will not be aggregated");
                }
                match zenoh::open(config).res() {
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
                                                            if gateway.zbus_aggregate {
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
                                                                if gateway.zbus_aggregate_and_split {
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
                                stdlib::sleep::sleep(1);
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
