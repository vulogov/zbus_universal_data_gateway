extern crate log;
use crate::cmd;
use crate::stdlib;
use serde_json::{Deserializer, Value};
use telegraf::*;

pub fn sender(c: &cmd::Cli, gateway: &cmd::Gateway)  {
    log::trace!("zbus_gateway_statsd_sender::run() reached");
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
                                                        // println!("BODY: {:?}", cmd::zbus_gateway_processor::zabbix_json_get_sub_subkey_raw(&zjson, "body".to_string(), "details".to_string(), "properties".to_string()));
                                                        let clock = match cmd::zbus_gateway_processor::zabbix_json_get_raw(&properties, "zabbix_clock".to_string()) {
                                                            Some(c) => c,
                                                            None => continue,
                                                        };
                                                        let ns = match cmd::zbus_gateway_processor::zabbix_json_get_raw(&properties, "zabbix_ns".to_string()) {
                                                            Some(c) => c,
                                                            None => continue,
                                                        };
                                                        if data.is_f64() {
                                                            let mdata = point!(itemkey.clone(), ("platform", c.platform_name.clone()) ("source", c.source.clone()), ("data", data.as_f64().unwrap()); stdlib::time::make_nanosecond_ts(clock.as_f64().unwrap(), ns.as_f64().unwrap()) as u64);
                                                            let _ = stream.write_point(&mdata);
                                                        } else if data.is_i64() {
                                                            let mdata = point!(itemkey.clone(), ("platform", c.platform_name.clone()) ("source", c.source.clone()), ("data", data.as_i64().unwrap()); stdlib::time::make_nanosecond_ts(clock.as_f64().unwrap(), ns.as_f64().unwrap()) as u64);
                                                            let _ = stream.write_point(&mdata);
                                                        } else {
                                                            log::debug!("Unsupported data type for {} = {:?}", &itemkey, &data);
                                                        }
                                                        // else if data.is_string() {
                                                        //     let value = format!("{}", data.as_str().unwrap());
                                                        //     let mdata = point!(itemkey.clone(), ("platform", c.platform_name.clone()) ("source", c.source.clone()), ("data", value); stdlib::time::make_nanosecond_ts(clock.as_f64().unwrap(), ns.as_f64().unwrap()) as u64);
                                                        //     let _ = stream.write_point(&mdata);
                                                        // }
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
                            break 'outside;
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
