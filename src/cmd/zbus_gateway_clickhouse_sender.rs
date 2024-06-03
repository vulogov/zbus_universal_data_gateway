extern crate log;
use crate::cmd;
use crate::stdlib;
use serde_json::{Deserializer, Value, json};
use reqwest;


pub fn sender(c: &cmd::Cli, gateway: &cmd::Gateway)  {
    log::trace!("zbus_gateway_clickhouse_sender::run() reached");
    let gateway = gateway.clone();
    let c       = c.clone();

    match stdlib::threads::THREADS.lock() {
        Ok(t) => {
            t.execute(move ||
            {
                log::debug!("CLICKHOUSE sender thread has been started");
                loop {

                    if ! stdlib::channel::pipe_is_empty_raw("out".to_string()) {
                        match stdlib::channel::pipe_pull("out".to_string()) {
                            Ok(res) => {
                                log::debug!("Received {} bytes by CLICKHOUSE processor", &res.len());
                                let vstream = Deserializer::from_str(&res).into_iter::<Value>();
                                let mut data = String::new();
                                for value in vstream {
                                    match value {
                                        Ok(zjson) => {
                                            let itemkey = match cmd::zbus_gateway_processor::zabbix_json_get_sub_subkey_raw(&zjson, "body".to_string(), "details".to_string(), "destination".to_string()) {
                                                Some(key) => format!("zbus/metric/{}/{}{}", &c.protocol_version, &c.platform_name, key.as_str().unwrap()),
                                                None => continue,
                                            };
                                            let properties = match cmd::zbus_gateway_processor::zabbix_json_get_sub_subkey_raw(&zjson, "body".to_string(), "details".to_string(), "properties".to_string()) {
                                                Some(d) => d,
                                                None => continue,
                                            };
                                            let clock = match cmd::zbus_gateway_processor::zabbix_json_get_raw(&properties, "zabbix_clock".to_string()) {
                                                Some(c) => c,
                                                None => continue,
                                            };
                                            let zabbix_host = match cmd::zbus_gateway_processor::zabbix_json_get_raw(&properties, "zabbix_host_name".to_string()) {
                                                Some(c) => c,
                                                None => continue,
                                            };
                                            let zabbix_item = match cmd::zbus_gateway_processor::zabbix_json_get_raw(&properties, "zabbix_item".to_string()) {
                                                Some(c) => c,
                                                None => continue,
                                            };
                                            let d = match cmd::zbus_gateway_processor::zabbix_json_get_sub_subkey_raw(&zjson, "body".to_string(), "details".to_string(), "details".to_string()) {
                                                Some(d) => d,
                                                None => continue,
                                            };
                                            let vdata = match cmd::zbus_gateway_processor::zabbix_json_get_raw(&d, "data".to_string()) {
                                                Some(d) => d,
                                                None => continue,
                                            };
                                            let itemtype = match cmd::zbus_gateway_processor::zabbix_json_get_raw(&d, "contentType".to_string()) {
                                                Some(d) => d.as_i64(),
                                                None => continue,
                                            };
                                            match itemtype.unwrap() {
                                                0 => {
                                                    let row = json!({
                                                        "ts":       clock.as_f64().unwrap() as u64,
                                                        "key":      itemkey.clone(),
                                                        "source":   c.source.clone(),
                                                        "zabbix_host":  zabbix_host.clone(),
                                                        "zabbix_item":  zabbix_item.clone(),
                                                        "data_type":    itemtype,
                                                        "data_float":   vdata,
                                                    });
                                                    data.push_str(&format!("INSERT INTO data FORMAT JSONEachRow {}\n", row));
                                                }
                                                3 => {
                                                    let row = json!({
                                                        "ts":       clock.as_f64().unwrap() as u64,
                                                        "key":      itemkey.clone(),
                                                        "source":   c.source.clone(),
                                                        "zabbix_host":  zabbix_host.clone(),
                                                        "zabbix_item":  zabbix_item.clone(),
                                                        "data_type":    itemtype,
                                                        "data_int":     vdata,
                                                    });
                                                    data.push_str(&format!("INSERT INTO data FORMAT JSONEachRow {}\n", row));
                                                }
                                                1 | 2 | 4 => {
                                                    let row = json!({
                                                        "ts":       clock.as_f64().unwrap() as u64,
                                                        "key":      itemkey.clone(),
                                                        "source":   c.source.clone(),
                                                        "zabbix_host":  zabbix_host.clone(),
                                                        "zabbix_item":  zabbix_item.clone(),
                                                        "data_type":    itemtype,
                                                        "data_str":     vdata,
                                                    });
                                                    data.push_str(&format!("INSERT INTO data FORMAT JSONEachRow {}\n", row));
                                                }
                                                _ => {
                                                    log::debug!("Unsupported data type for {} = {:?}", &itemkey, &data);
                                                }
                                            }
                                        }
                                        Err(err) => {
                                            log::error!("Error converting JSON: {:?}", err);
                                        }
                                    }
                                    log::debug!("End of JSON");
                                }
                                log::debug!("End of JSON series. Submitting data to CLICKHOUSE");
                                log::debug!("Connected to CLICKHOUSE at {}database={}", &gateway.clickhouse_connect, &gateway.clickhouse_database);
                                match reqwest::blocking::Client::new()
                                            .post(format!("{}database={}", gateway.clickhouse_connect, gateway.clickhouse_database))
                                            .body(data)
                                            .send() {
                                    Ok(res) => {
                                        if res.status() != reqwest::StatusCode::OK {
                                            log::error!("Error submitting CLICKHOUSE data: {}", res.status());
                                        }
                                    }
                                    Err(err) => {
                                        log::error!("Error connecting to CLICKHOUSE: {}", err);
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
