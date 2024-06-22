extern crate log;
use crate::cmd;
use crate::stdlib;
use serde_json::{json, Deserializer, Value};

pub fn processor(c: &cmd::Cli, alerts: &cmd::Alerts)  {
    log::debug!("zbus_alerts_processor::run() reached");
    let c = c.clone();
    let alerts = alerts.clone();
    match stdlib::threads::THREADS.lock() {
        Ok(t) => {
            t.execute(move ||
            {
                log::debug!("ALERTS PROCESSOR thread has been started");
                loop {
                    if ! stdlib::channel::pipe_is_empty_raw("in".to_string()) {
                        match stdlib::channel::pipe_pull("in".to_string()) {
                            Ok(res) => {
                                log::debug!("Received {} bytes by processor", &res.len());
                                let stream = Deserializer::from_str(&res).into_iter::<Value>();
                                for value in stream {
                                    match value {
                                        Ok(zjson) => {
                                            if ! zjson.is_object() {
                                                log::error!("Received JSON is not an object: {}", &zjson);
                                                continue;
                                            }
                                            let value = match cmd::zbus_gateway_processor::zabbix_json_get_raw(&zjson, "value".to_string()) {
                                                Some(value) => value,
                                                None => continue,
                                            };
                                            if ! value.is_i64() {
                                                log::error!("Alert value is not an integer");
                                                continue;
                                            }
                                            let ivalue = match value.as_i64() {
                                                Some(ivalue) => ivalue,
                                                None => continue,
                                            };
                                            let id = match ivalue {
                                                0 => {
                                                    match stdlib::alerts::resolve_alert(zjson.clone()) {
                                                        Some(id) => id,
                                                        None => continue,
                                                    }
                                                }
                                                1 => {
                                                    log::debug!("Adding alert");
                                                    match stdlib::alerts::add_alert(zjson.clone()) {
                                                        Some(id) => id,
                                                        None => continue,
                                                    }
                                                }
                                                _ => continue,
                                            };
                                            log::trace!("Alertid: {:?} {:?}", &id, &zjson);
                                            let data = json!({
                                                "headers": {
                                                    "messageType":      "event",
                                                    "route":            c.route.clone(),
                                                    "streamName":       c.platform_name.clone(),
                                                    "cultureCode":      null,
                                                    "version":          c.protocol_version.clone(),
                                                    "encryptionAlgorithm":      null,
                                                    "compressionAlgorithm":     null,
                                                },
                                                "body": {
                                                    "details": {
                                                        "origin":       alerts.source.clone(),
                                                        "destination":  format!("/{}", alerts.zbus_key),
                                                        "properties":   {
                                                            "zabbix_clock":     cmd::zbus_gateway_processor::zabbix_json_get(&zjson, "clock".to_string()),
                                                            "zabbix_ns":        cmd::zbus_gateway_processor::zabbix_json_get(&zjson, "ns".to_string()),
                                                            "zabbix_host_name": cmd::zbus_gateway_processor::zabbix_json_get(&zjson, "hosts".to_string()),
                                                            "zabbix_eventid":   cmd::zbus_gateway_processor::zabbix_json_get(&zjson, "eventid".to_string()),
                                                            "name":             cmd::zbus_gateway_processor::zabbix_json_get(&zjson, "name".to_string()),
                                                            "tags":             cmd::zbus_gateway_processor::zabbix_json_get(&zjson, "tags".to_string()),
                                                            "groups":           cmd::zbus_gateway_processor::zabbix_json_get(&zjson, "groups".to_string()),
                                                        },
                                                        "details":  {
                                                            "detailType":   "",
                                                            "contentType":  3,
                                                            "data":         value,
                                                        }
                                                    }
                                                },
                                                "id": id,
                                            });
                                            match &alerts.script {
                                                Some(_) => {
                                                    stdlib::channel::pipe_push("filter".to_string(), data.to_string());
                                                }
                                                None => {

                                                        stdlib::channel::pipe_push("out".to_string(), data.to_string());
                                                }
                                            }
                                        }
                                        Err(err) => {
                                            log::error!("Error converting JSON: {:?}", err);
                                        }
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
