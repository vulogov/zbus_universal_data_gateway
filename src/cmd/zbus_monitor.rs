extern crate log;
use crate::cmd;
use std::io::{stdin, Read};
use std::str::FromStr;
use zenoh::prelude::sync::*;
use zenoh::config::{Config, ConnectConfig, ListenConfig, EndPoint, WhatAmI};

pub fn run(_c: &cmd::Cli, monitor: &cmd::Monitor)  {
    log::trace!("zbus_monitor::run() reached");
    let mut config =  Config::default();

    if monitor.zbus_disable_multicast_scout.clone() {
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
    match EndPoint::from_str(&monitor.zbus_connect) {
        Ok(zconn) => {
            log::debug!("ZENOH bus set to: {:?}", &zconn);
            let _ = config.set_connect(ConnectConfig::new(vec![zconn]).unwrap());
        }
        Err(err) => {
            log::error!("Failure in parsing connect address: {:?}", err);
            return;
        }
    }
    match EndPoint::from_str(&monitor.zbus_listen) {
        Ok(zlisten) => {
            log::debug!("ZENOH listen set to: {:?}", &zlisten);
            let _ = config.set_listen(ListenConfig::new(vec![zlisten]).unwrap());
        }
        Err(_) => {
            log::debug!("ZENOH listen set to default");
        }
    }
    if monitor.zbus_set_connect_mode {
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
    match zenoh::open(config).res() {
        Ok(session) => {
            log::debug!("Connection to ZENOH bus succesful");
            log::debug!("Telemetry key is: {}", &monitor.zbus_key);
            match session.declare_subscriber(&monitor.zbus_key)
                    .callback_mut(move |sample| {
                        let slices = &sample.value.payload.contiguous();
                        match std::str::from_utf8(slices) {
                            Ok(data) => {
                                match serde_json::from_str::<serde_json::Value>(&data) {
                                    Ok(zjson) => {
                                        println!("{}", &zjson.to_string());
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
                    for byte in stdin().bytes() {
                        match byte {
                            Ok(b'q') => break,
                            _ => std::thread::yield_now(),
                        }
                    }
                }
                Err(err) => {
                    log::error!("Telemetry subscribe for key {} failed: {:?}", &monitor.zbus_key, err);
                    return;
                }
            }
            let _ = session.close();
            log::debug!("Connection to ZENOH bus is closed");
        }
        Err(err) => {
            log::error!("Error connecting to ZENOH bus: {:?}", err);
        }
    }
}
