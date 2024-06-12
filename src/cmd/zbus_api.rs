extern crate log;
use crate::cmd;
use crate::stdlib;
use serde_json::{Value};
use std::collections::BTreeMap;
use std::sync::Mutex;
use lazy_static::lazy_static;
use std::str::FromStr;
use zenoh::prelude::sync::*;
use zenoh::config::{Config, ConnectConfig, ListenConfig, EndPoint, WhatAmI};

lazy_static! {
    static ref SAMPLES: Mutex<BTreeMap<String, cmd::zbus_value_sampler::ValueSampler>> = {
        let m: Mutex<BTreeMap<String, cmd::zbus_value_sampler::ValueSampler>> = Mutex::new(BTreeMap::new());
        m
    };
}

pub fn create_metric(n: String) {
    let mut q = SAMPLES.lock().unwrap();
    q.insert(n.to_string(), cmd::zbus_value_sampler::ValueSampler::init());
    drop(q);
}

pub fn push_metric(k: String, v: Value) {
    let mut s = SAMPLES.lock().unwrap();
    if ! s.contains_key(&k) {
        let mut sample = cmd::zbus_value_sampler::ValueSampler::init();
        sample.set(v);
        s.insert(k.clone(), sample);
    } else {
        let sample = s.get_mut(&k).unwrap();
        sample.set(v);
    }
    drop(s);
}

pub fn get_metric(k: String) -> Option<cmd::zbus_value_sampler::ValueSampler> {
    let s = SAMPLES.lock().unwrap();
    if ! s.contains_key(&k) {
        drop(s);
        return None;
    }
    let sample = s.get(&k).unwrap().clone();
    drop(s);
    return Some(sample);
}

pub fn get_keys() -> Vec<String> {
    let mut res: Vec<String> = Vec::new();
    let s = SAMPLES.lock().unwrap();
    for k in s.keys() {
        res.push(k.clone());
    }
    drop(s);
    return res;
}

pub fn run(c: &cmd::Cli, apicli: &cmd::Api)  {
    log::debug!("zbus_api::run() reached");
    let mut config =  Config::default();
    let c = c.clone();

    cmd::zbus_api_rpc::run(&c, apicli);

    if apicli.zbus_disable_multicast_scout.clone() {
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
    match EndPoint::from_str(&apicli.zbus_connect) {
        Ok(zconn) => {
            log::debug!("ZENOH bus set to: {:?}", &zconn);
            let _ = config.set_connect(ConnectConfig::new(vec![zconn]).unwrap());
        }
        Err(err) => {
            log::error!("Failure in parsing connect address: {:?}", err);
            return;
        }
    }
    match EndPoint::from_str(&apicli.zbus_listen) {
        Ok(zlisten) => {
            log::debug!("ZENOH listen set to: {:?}", &zlisten);
            let _ = config.set_listen(ListenConfig::new(vec![zlisten]).unwrap());
        }
        Err(_) => {
            log::debug!("ZENOH listen set to default");
        }
    }
    if apicli.zbus_set_connect_mode {
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
                log::debug!("Telemetry key is: {}", &apicli.zbus_key);
                match session.declare_subscriber(&apicli.zbus_key)
                        .callback_mut(move |sample| {
                            let slices = &sample.value.payload.contiguous();
                            match std::str::from_utf8(slices) {
                                Ok(data) => {
                                    match serde_json::from_str::<serde_json::Value>(&data) {
                                        Ok(zjson) => {
                                            if ! zjson.is_object() {
                                                log::error!("Received JSON is not an object: {}", &zjson);
                                                return;
                                            }
                                            let itemkey = match cmd::zbus_gateway_processor::zabbix_json_get_sub_subkey_raw(&zjson, "body".to_string(), "details".to_string(), "destination".to_string()) {
                                                Some(key) => key.as_str().unwrap().to_string(),
                                                None => return,
                                            };
                                            let d = match cmd::zbus_gateway_processor::zabbix_json_get_sub_subkey_raw(&zjson, "body".to_string(), "details".to_string(), "details".to_string()) {
                                                Some(d) => d,
                                                None => return,
                                            };
                                            let data = match cmd::zbus_gateway_processor::zabbix_json_get_raw(&d, "data".to_string()) {
                                                Some(d) => d,
                                                None => return,
                                            };
                                            push_metric(itemkey.clone(), data);
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
                        log::error!("Telemetry subscribe for key {} failed: {:?}", &apicli.zbus_key, err);
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
}
