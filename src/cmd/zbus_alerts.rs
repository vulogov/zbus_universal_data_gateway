extern crate log;
use crate::cmd;
use crate::stdlib;
use std::str::FromStr;
use std::path::Path;
use zenoh::config::{Config, ConnectConfig, ListenConfig, EndPoint, WhatAmI};

pub fn run(c: &cmd::Cli, alerts: &cmd::Alerts)  {
    log::debug!("zbus_alerts::run() reached");

    let mut config =  Config::default();
    if alerts.zbus_disable_multicast_scout.clone() {
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
    match EndPoint::from_str(&alerts.zbus_connect) {
        Ok(zconn) => {
            log::debug!("ZENOH bus set to: {:?}", &zconn);
            let _ = config.set_connect(ConnectConfig::new(vec![zconn]).unwrap());
        }
        Err(err) => {
            log::error!("Failure in parsing connect address: {:?}", err);
            return;
        }
    }
    match EndPoint::from_str(&alerts.zbus_listen) {
        Ok(zlisten) => {
            log::debug!("ZENOH listen set to: {:?}", &zlisten);
            let _ = config.set_listen(ListenConfig::new(vec![zlisten]).unwrap());
        }
        Err(_) => {
            log::debug!("ZENOH listen set to default");
        }
    }
    if alerts.zbus_set_connect_mode {
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

    match &alerts.script {
        Some(fname) => {
            if Path::new(&fname).exists() {
                log::debug!("Filtering and transformation enabled");
                cmd::zbus_alerts_processor_filter::processor(c, alerts);
                cmd::zbus_alerts_processor_transformation::processor(c, alerts);
            } else {
                log::error!("Script not found processing disabled");
                return;
            }
        }
        None => log::debug!("Filtering disabled"),
    }

    cmd::zbus_alerts_zabbix::catcher(c, alerts);
    cmd::zbus_alerts_processor::processor(c, alerts);
    cmd::zbus_thread_zbus_sender::sender(c, config, true, false, alerts.zbus_key.clone());


    stdlib::threads::wait_all();
}
