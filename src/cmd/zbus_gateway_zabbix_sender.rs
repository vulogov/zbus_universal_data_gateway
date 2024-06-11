extern crate log;
use crate::cmd;
use crate::stdlib;
use serde_json::{Deserializer, Value};

pub fn sender(_c: &cmd::Cli, gateway: &cmd::Gateway)  {
    log::trace!("zbus_gateway_zabbix_sender::run() reached");
    let gateway = gateway.clone();

    log::debug!("Will be connecting to Zabbix Sender service at: {}", &gateway.zabbix_sender_connect);

    match stdlib::threads::THREADS.lock() {
        Ok(t) => {
            t.execute(move ||
            {
                log::debug!("ZABBIX SENDER sender thread has been started");
                loop {
                    if ! stdlib::channel::pipe_is_empty_raw("out".to_string()) {
                        match stdlib::channel::pipe_pull("out".to_string()) {
                            Ok(res) => {
                                log::debug!("Received {} bytes by ZABBIX processor", &res.len());
                                let stream = Deserializer::from_str(&res).into_iter::<Value>();
                                for value in stream {
                                    match value {
                                        Ok(zjson) => {
                                            let d = match cmd::zbus_gateway_processor::zabbix_json_get_sub_subkey_raw(&zjson, "body".to_string(), "details".to_string(), "details".to_string()) {
                                                Some(d) => d,
                                                None => continue,
                                            };
                                            let orig = match cmd::zbus_gateway_processor::zabbix_json_get_sub_subkey_raw(&zjson, "body".to_string(), "details".to_string(), "origin".to_string()) {
                                                Some(d) => d,
                                                None => continue,
                                            };
                                            let p = match cmd::zbus_gateway_processor::zabbix_json_get_sub_subkey_raw(&zjson, "body".to_string(), "details".to_string(), "properties".to_string()) {
                                                Some(p) => p,
                                                None => continue,
                                            };
                                            let data = match cmd::zbus_gateway_processor::zabbix_json_get_raw(&d, "data".to_string()) {
                                                Some(d) => d,
                                                None => continue,
                                            };
                                            let zitemkey = match cmd::zbus_gateway_processor::zabbix_json_get_raw(&p, "zabbix_item".to_string()) {
                                                Some(d) => d,
                                                None => continue,
                                            };
                                            let data = serde_json::json!({
                                                "request": "sender data",
                                                "data": [{
                                                    "host":     orig,
                                                    "key":      zitemkey,
                                                    "value":    data.clone(),
                                                },]
                                            });
                                            match stdlib::zabbix::zabbix_sender(gateway.zabbix_sender_connect.clone(), data) {
                                                Ok(res) => {
                                                    log::debug!("Received from Zabbix: {:?}", &res);
                                                },
                                                Err(err) => {
                                                    log::error!("{}", err);
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
            });
            drop(t);
        }
        Err(err) => {
            log::error!("Error accessing Thread Manager: {:?}", err);
            return;
        }
    }
}
