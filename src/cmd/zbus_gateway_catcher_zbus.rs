extern crate log;
use crate::cmd;
use crate::stdlib;
use std::str::FromStr;
use zenoh::prelude::sync::*;
use zenoh::config::{Config, ConnectConfig, ListenConfig, EndPoint, WhatAmI};

pub fn catcher(c: &cmd::Cli, gateway: &cmd::Gateway)  {
    log::trace!("zbus_gateway_catcher_zbus::run() reached");
    let subscribe_key = format!("zbus/metric/{}/{}/{}", &c.protocol_version, &c.platform_name, &gateway.zbus_subscribe_key);
    // let subscribe_key = gateway.zbus_subscribe_key.clone();
    let gateway = gateway.clone();

    match stdlib::threads::THREADS.lock() {
        Ok(t) => {
            t.execute(move ||
            {
                log::debug!("ZBUS catcher thread has been started");
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
                match EndPoint::from_str(&gateway.zbus_catcher_connect) {
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

                'outside: loop {
                    match zenoh::open(config.clone()).res() {
                        Ok(session) => {
                            log::debug!("Connection to ZENOH bus succesful");
                            log::debug!("ZBUS catcher subscribing to: {}", &subscribe_key);
                            match session.declare_subscriber(&subscribe_key)
                                    .callback_mut(move |sample| {
                                        let slices = &sample.value.payload.contiguous();
                                        match std::str::from_utf8(slices) {
                                            Ok(data) => {
                                                match serde_json::from_str::<serde_json::Value>(&data) {
                                                    Ok(zjson) => {
                                                        log::debug!("ZBUS catcher received {} bytes", &data.len());
                                                        stdlib::channel::pipe_push("in".to_string(), zjson.to_string());
                                                    }
                                                    Err(err) => {
                                                        log::error!("Error while converting JSON data from ZENOH bus: {:?}", err);
                                                    }
                                                }
                                            }
                                            Err(err) => {
                                                log::error!("Error while extracting data from ZENOH bus: {:?}", err);
                                            }
                                        }
                                    })
                                    .res() {
                                Ok(_) => {
                                    let receiver = match zenoh::scout(WhatAmI::Peer, config.clone())
                                        .res() {
                                            Ok(receiver) => receiver,
                                            Err(err) => {
                                                log::error!("ZBUS scout had failed: {}", err);
                                                stdlib::sleep::sleep(5);
                                                continue 'outside;
                                            }
                                        };
                                    log::debug!("Running ZBUS scout to detect the health of connection");
                                    while let Ok(hello) = receiver.recv() {
                                        log::trace!("ZBUS catcher received: {}", hello);
                                        std::thread::yield_now();
                                    }
                                }
                                Err(err) => {
                                    log::error!("Telemetry subscribe for key {} failed: {:?}", &subscribe_key, err);
                                    stdlib::sleep::sleep(5);
                                    continue 'outside;
                                }
                            }
                            let _ = session.close();
                            log::debug!("Connection to ZENOH bus is closed");
                            stdlib::sleep::sleep(5);
                            continue 'outside;
                        }
                        Err(err) => {
                            log::error!("Error connecting to ZENOH bus: {:?}", err);
                            stdlib::sleep::sleep(5);
                            continue 'outside;
                        }
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
