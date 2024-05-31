extern crate log;
use crate::cmd;
use crate::stdlib;
use serde_json::{Deserializer, Value};
use statsd::Client;

pub fn sender(c: &cmd::Cli, gateway: &cmd::Gateway)  {
    log::trace!("zbus_gateway_statsd_sender::run() reached");
    let gateway = gateway.clone();
    let c       = c.clone();

    match stdlib::threads::THREADS.lock() {
        Ok(t) => {
            t.execute(move ||
            {
                log::debug!("STATSD sender thread has been started");
                'outside: loop {
                    match Client::new(&gateway.statsd_connect, &c.platform_name) {
                        Ok(stream) => {
                            loop {
                                if ! stdlib::channel::pipe_is_empty_raw("out".to_string()) {
                                    match stdlib::channel::pipe_pull("out".to_string()) {
                                        Ok(res) => {
                                            log::debug!("Received {} bytes by STATSD processor", &res.len());
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
                                                        let data = match cmd::zbus_gateway_processor::zabbix_json_get_raw(&d, "data".to_string()) {
                                                            Some(d) => d,
                                                            None => continue,
                                                        };
                                                        if data.is_f64() {
                                                            stream.gauge(&itemkey, data.as_f64().unwrap());
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
                            log::error!("Error connecting to MQTT: {}", err);
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
