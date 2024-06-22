extern crate log;
use std::collections::btree_map::BTreeMap;
use lazy_static::lazy_static;
use std::sync::Mutex;
use nanoid::nanoid;
use crate::cmd;
use serde_json::{Value};

#[allow(dead_code)]
#[derive(Debug, Clone)]
struct AlertInfo {
    pub eid:        i64,
    pub id:         String,
    pub data:       Value,
}

lazy_static! {
    static ref ALERTS: Mutex<BTreeMap<i64, AlertInfo>> = {
        let m: Mutex<BTreeMap<i64, AlertInfo>> = Mutex::new(BTreeMap::new());
        m
    };
}

pub fn add_alert(data: Value) -> Option<String> {
    let eventid = match cmd::zbus_gateway_processor::zabbix_json_get_raw(&data, "eventid".to_string()) {
        Some(value) => value,
        None => return None,
    };
    if ! eventid.is_i64() {
        return None;
    }
    let ieventid = match eventid.as_i64() {
        Some(ieventid) => ieventid,
        None => return None,
    };
    let ainfo = AlertInfo{
        data:   data.clone(),
        id:     nanoid!(),
        eid:    ieventid,
    };
    let mut a = ALERTS.lock().unwrap();
    a.insert(ieventid, ainfo.clone());
    drop(a);
    return Some(ainfo.id.clone());
}

pub fn resolve_alert(data: Value) -> Option<String> {
    let eventid = match cmd::zbus_gateway_processor::zabbix_json_get_raw(&data, "p_eventid".to_string()) {
        Some(value) => value,
        None => return None,
    };
    if ! eventid.is_i64() {
        return None;
    }
    let ieventid = match eventid.as_i64() {
        Some(ieventid) => ieventid,
        None => return None,
    };
    let mut a = ALERTS.lock().unwrap();
    match a.remove(&ieventid) {
        Some(ainfo) => {
            drop(a);
            return Some(ainfo.id.clone());
        }
        None => {
            drop(a);
            return None;
        }
    }
}

pub fn alerts_init() {
    log::debug!("Running STDLIB::alerts init");
    let a = ALERTS.lock().unwrap();
    drop(a);
}
