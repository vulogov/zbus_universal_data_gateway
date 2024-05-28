extern crate log;
use crate::cmd;
use crate::stdlib;
use nanoid::nanoid;
use std::sync::Mutex;
use std::time::Duration;
use lazy_static::lazy_static;
use timedmap::TimedMap;
use etime::Etime;
use serde_json::{json, from_str, Deserializer, Value};

lazy_static! {
    static ref ITEMS: Mutex<TimedMap<String, String>> = {
        let e: Mutex<TimedMap<String, String>>  = Mutex::new(TimedMap::new());
        e
    };
}

fn zabbix_json_get(data: &Value, key: String) -> Value {
    match data.get(key) {
        Some(value) => {
            return value.clone();
        }
        None => {
            return json!(null);
        }
    }
}

fn zabbix_json_get_raw(data: &Value, key: String) -> Option<Value> {
    let m = match data.as_object() {
        Some(m) => m,
        None => {
            log::error!("Failure to convert to MAP: {} for {}", &data, &key);
            return None;
        }
    };
    if m.contains_key(&key) {
        return Some(data.get(key.clone()).unwrap().clone());
    }
    None
}

fn zabbix_json_get_subkey(data: &Value, key: String, subkey: String) -> Value {
    let m = match data.as_object() {
        Some(m) => m,
        None => {
            log::error!("Failure to convert to MAP: {}", &data);
            return json!(null);
        }
    };
    if m.contains_key(&key) {
        match data.get(key) {
            Some(value) => {
                return zabbix_json_get(value, subkey);
            }
            None => {
                return json!(null);
            }
        }
    }
    json!(null)
}

fn zabbix_json_get_subkey_raw(data: &Value, key: String, subkey: String) -> Option<Value> {
    let m = match data.as_object() {
        Some(m) => Some(m),
        None => {
            log::error!("Failure to convert to MAP: {}", &data);
            return None;
        }
    };
    if m?.contains_key(&key) {
        match data.get(key) {
            Some(value) => {
                return zabbix_json_get_raw(value, subkey);
            }
            None => {
                return None;
            }
        }
    }
    return None
}

pub fn zabbix_json_get_sub_subkey_raw(data: &Value, key: String, subkey: String, subsubkey: String) -> Option<Value> {
    let m = match zabbix_json_get_subkey_raw(data, key, subkey) {
        Some(m) => m,
        None => return None,
    };
    match zabbix_json_get_raw(&m, subsubkey) {
        Some(v) => { return Some(v); }
        None => {}
    }
    return None
}

fn zabbix_get_item_info(c: &cmd::Cli, gateway: &cmd::Gateway, itemid: String) -> Option<Value> {
    let i = ITEMS.lock().unwrap();
    match i.get(&itemid) {
        Some(val) => {
            drop(i);
            log::debug!("Getting item {} from cache", &itemid);
            return Some(from_str(&val).unwrap());
        }
        None => {
            match zabbix_get_item_info_zabbix(c, gateway, itemid.clone()) {
                Some(val) => {
                    i.insert(itemid.clone(), val.to_string(), Duration::from_secs(c.item_cache_timeout.into()));
                    drop(i);
                    log::debug!("Storing item {} to cache", &itemid);
                    return Some(val);
                }
                None => {
                    return None;
                }
            }
        }
    }

}

fn zabbix_get_item_info_zabbix(c: &cmd::Cli, gateway: &cmd::Gateway, itemid: String) -> Option<Value> {
    match reqwest::blocking::Client::new()
                .post(format!("{}/api_jsonrpc.php", c.zabbix_api))
                .bearer_auth(&gateway.zabbix_token)
                .json(&serde_json::json!({
                    "jsonrpc": "2.0",
                    "method": "item.get",
                    "id": 1,
                    "params": {
                        "itemids":      itemid,
                        "templated":    false,
                    }
                }))
                .send() {
        Ok(res) => {
            let jres: serde_json::Value = match res.json() {
                Ok(jres) => jres,
                Err(err) => {
                    log::error!("Error in item.get: {:?}", err);
                    return None;
                }
            };
            match jres.get("result") {
                Some(result) => {
                    match result.as_array() {
                        Some(item_values) => {
                            if item_values.len() == 0 {
                                log::error!("Zabbix item.get returned empty array for {}", &itemid);
                                return None;
                            } else {
                                return Some(result[0].clone());
                            }
                        }
                        None => {
                            log::error!("Zabbix item.get returned non-array");
                            return None;
                        }
                    }
                }
                None => {
                    println!("Error in: {:?}", &jres);
                }
            }
        }
        Err(err) => {
            log::error!("Error in item.get: {:?}", err);
        }
    }
    None
}

fn zabbix_get_item_key(c: &cmd::Cli, gateway: &cmd::Gateway, itemid: String) -> Option<String> {
    match zabbix_get_item_info(c, gateway, itemid) {
        Some(result) => {
            match zabbix_json_get_raw(&result, "key_".to_string()) {
                Some(itemkey) => {
                    return Some(itemkey.to_string());
                }
                None => {
                    log::error!("Zabbix item.get returned struct that do not have key_: {}", &result);
                    return None;
                }
            }
        }
        None => {
            return None;
        }
    }
}

pub fn processor(c: &cmd::Cli, gateway: &cmd::Gateway)  {
    log::trace!("zbus_gateway_processor::run() reached");
    let c = c.clone();
    let gateway = gateway.clone();
    match stdlib::threads::THREADS.lock() {
        Ok(t) => {
            t.execute(move ||
            {
                log::debug!("PROCESSOR thread has been started");
                loop {
                    if ! stdlib::channel::pipe_is_empty_raw("in".to_string()) {
                        match stdlib::channel::pipe_pull("in".to_string()) {
                            Ok(res) => {
                                log::debug!("Received {} bytes by processor", &res.len());
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
                                            let itemkey = match zabbix_json_get_raw(&zjson, "itemid".to_string()) {
                                                Some(jitemid) => match zabbix_get_item_key(&c, &gateway, jitemid.to_string()) {
                                                    Some(zkey) => zkey,
                                                    None => continue,

                                                },
                                                None => {
                                                    log::error!("Zabbix JSON is malformed. No itemid key");
                                                    continue;
                                                }
                                            };
                                            let zbus_itemkey = match cmd::zabbix_lib::zabbix_key_to_zenoh(itemkey.clone()) {
                                                Some(zbus_key) => zbus_key,
                                                None => continue,
                                            };
                                            let data = json!({
                                                "headers": {
                                                    "messageType":      "telemetry",
                                                    "route":            c.route.clone(),
                                                    "streamName":       c.platform_name.clone(),
                                                    "cultureCode":      null,
                                                    "version":          c.protocol_version.clone(),
                                                    "encryptionAlgorithm":      null,
                                                    "compressionAlgorithm":     null,
                                                },
                                                "body": {
                                                    "details": {
                                                        "origin":       zabbix_json_get_subkey(&zjson, "host".to_string(), "host".to_string()),
                                                        "destination":  zbus_itemkey.clone(),
                                                        "properties":   {
                                                            "zabbix_clock":     zabbix_json_get(&zjson, "clock".to_string()),
                                                            "zabbix_ns":        zabbix_json_get(&zjson, "ns".to_string()),
                                                            "zabbix_host_name": zabbix_json_get_subkey(&zjson, "host".to_string(), "name".to_string()),
                                                            "zabbix_itemid":    zabbix_json_get(&zjson, "itemid".to_string()),
                                                            "zabbix_item":      itemkey.clone(),
                                                            "name":             zabbix_json_get(&zjson, "name".to_string()),
                                                            "tags":             zabbix_json_get(&zjson, "tags".to_string()),

                                                        },
                                                        "details":  {
                                                            "detailType":   "",
                                                            "contentType":  zabbix_json_get(&zjson, "type".to_string()),
                                                            "data":         zabbix_json_get(&zjson, "value".to_string()),
                                                        }
                                                    }
                                                },
                                                "id": nanoid!(),
                                            });
                                            if ! gateway.group.none {
                                                stdlib::channel::pipe_push("out".to_string(), data.to_string());
                                            }
                                            // stdlib::channel::pipe_push("out".to_string(), zjson.to_string());
                                        }
                                        Err(err) => {
                                            log::error!("Error converting JSON: {:?}", err);
                                        }
                                    }
                                }
                                let elapsed = e.toc().as_secs_f32();
                                log::debug!("Elapsed time for processing: {} seconds", elapsed);
                                if gateway.telemetry_monitor_elapsed {
                                    let data = cmd::zbus_json::generate_json_telemetry(&c, "/zbus/udg/elapsed".to_string(), "Elapsed time for JSON batch processing".to_string(), 3, json!(elapsed));
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
