extern crate log;
use crate::cmd;
use crate::stdlib;
use serde_json::{Deserializer, Value};
use telegraf::*;
use telegraf::protocol::{Field, FieldData};

pub fn sender(c: &cmd::Cli, gateway: &cmd::Gateway)  {
    log::trace!("zbus_gateway_telegraf_sender::run() reached");
    let gateway = gateway.clone();
    let c       = c.clone();

    match stdlib::threads::THREADS.lock() {
        Ok(t) => {
            t.execute(move ||
            {
                log::debug!("TELEGRAF sender thread has been started");
                'outside: loop {
                    match Client::new(&gateway.telegraf_connect) {
                        Ok(mut stream) => {
                            log::debug!("Connected to TELEGRAF at {}", &gateway.telegraf_connect);
                            loop {
                                if ! stdlib::channel::pipe_is_empty_raw("out".to_string()) {
                                    match stdlib::channel::pipe_pull("out".to_string()) {
                                        Ok(res) => {
                                            log::debug!("Received {} bytes by TELEGRAF processor", &res.len());
                                            let vstream = Deserializer::from_str(&res).into_iter::<Value>();
                                            for value in vstream {
                                                match value {
                                                    Ok(zjson) => {
                                                        let itemkey = match cmd::zbus_gateway_processor::zabbix_json_get_sub_subkey_raw(&zjson, "body".to_string(), "details".to_string(), "destination".to_string()) {
                                                            Some(key) => format!("zbus/metric/{}/{}{}", &c.protocol_version, &c.platform_name, key.as_str().unwrap()),
                                                            None => continue,
                                                        };
                                                        let d = match cmd::zbus_gateway_processor::zabbix_json_get_sub_subkey_raw(&zjson, "body".to_string(), "details".to_string(), "details".to_string()) {
                                                            Some(d) => d,
                                                            None => continue,
                                                        };
                                                        let properties = match cmd::zbus_gateway_processor::zabbix_json_get_sub_subkey_raw(&zjson, "body".to_string(), "details".to_string(), "properties".to_string()) {
                                                            Some(d) => d,
                                                            None => continue,
                                                        };
                                                        let data = match cmd::zbus_gateway_processor::zabbix_json_get_raw(&d, "data".to_string()) {
                                                            Some(d) => d,
                                                            None => continue,
                                                        };
                                                        let itemtype = match cmd::zbus_gateway_processor::zabbix_json_get_raw(&d, "contentType".to_string()) {
                                                            Some(d) => d.as_i64(),
                                                            None => continue,
                                                        };
                                                        // println!("BODY: {:?}", cmd::zbus_gateway_processor::zabbix_json_get_sub_subkey_raw(&zjson, "body".to_string(), "details".to_string(), "properties".to_string()));
                                                        let clock = match cmd::zbus_gateway_processor::zabbix_json_get_raw(&properties, "zabbix_clock".to_string()) {
                                                            Some(c) => c,
                                                            None => continue,
                                                        };
                                                        let ns = match cmd::zbus_gateway_processor::zabbix_json_get_raw(&properties, "zabbix_ns".to_string()) {
                                                            Some(c) => c,
                                                            None => continue,
                                                        };
                                                        let zabbix_host = match cmd::zbus_gateway_processor::zabbix_json_get_raw(&properties, "zabbix_host_name".to_string()) {
                                                            Some(c) => c,
                                                            None => continue,
                                                        };
                                                        let mut mdata = Point::new(
                                                            String::from(itemkey.clone()),
                                                            vec![
                                                                (String::from("platform"),      String::from(c.platform_name.clone())),
                                                                (String::from("source"),        String::from(c.source.clone())),
                                                                (String::from("zabbix_host"),   String::from(zabbix_host.to_string())),
                                                            ],
                                                            vec![
                                                                (String::from("zabbix_type"), Box::new(itemtype.unwrap())),
                                                            ],
                                                            Some(stdlib::time::make_nanosecond_ts(clock.as_f64().unwrap(), ns.as_f64().unwrap()) as u64),
                                                        );
                                                        match itemtype.unwrap() {
                                                            0 => {
                                                                mdata.fields.push(
                                                                    Field {
                                                                        name:   String::from("data"),
                                                                        value:  FieldData::Float(data.as_f64().unwrap()),
                                                                    }
                                                                );
                                                            }
                                                            2 => {
                                                                mdata.fields.push(
                                                                    Field {
                                                                        name:   String::from("data"),
                                                                        value:  FieldData::Str(data.as_str().unwrap().to_string()),
                                                                    }
                                                                );
                                                            }
                                                            3 => {
                                                                mdata.fields.push(
                                                                    Field {
                                                                        name:   String::from("data"),
                                                                        value:  FieldData::Number(data.as_i64().unwrap()),
                                                                    }
                                                                );
                                                            }
                                                            1 | 4 => {
                                                                mdata.fields.push(
                                                                    Field {
                                                                        name:   String::from("data"),
                                                                        value:  FieldData::Str(escape_string::escape(data.as_str().unwrap()).to_string()),
                                                                    }
                                                                );
                                                            }
                                                            _ => {
                                                                log::debug!("Unsupported data type for {} = {:?}", &itemkey, &data);
                                                            }
                                                        }
                                                        match stream.write_point(&mdata) {
                                                            Ok(_) => {}
                                                            Err(err) => {
                                                                log::error!("Error submitting metrics to TELEGRAF: {}", err);
                                                                continue 'outside;
                                                            }
                                                        }
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
                            log::error!("Error connecting to TELEGRAF: {}", err);
                            stdlib::sleep::sleep(1);
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
