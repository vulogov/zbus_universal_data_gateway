extern crate log;
use crate::cmd;
use crate::stdlib;
use serde_json::{Deserializer, Value};


pub fn sender(c: &cmd::Cli, gateway: &cmd::Gateway)  {
    log::trace!("zbus_gateway_rhai_sender::run() reached");
    let gateway = gateway.clone();
    let c = c.clone();
    match stdlib::threads::THREADS.lock() {
        Ok(t) => {
            t.execute(move ||
            {
                log::debug!("RHAI sender thread has been started");
                loop {

                    if ! stdlib::channel::pipe_is_empty_raw("out".to_string()) {
                        match stdlib::channel::pipe_pull("out".to_string()) {
                            Ok(res) => {
                                log::debug!("Received {} bytes by RHAI processor", &res.len());
                                let vstream = Deserializer::from_str(&res).into_iter::<Value>();
                                for value in vstream {
                                    match value {
                                        Ok(zjson) => {
                                            let itemkey = match cmd::zbus_gateway_processor::zabbix_json_get_sub_subkey_raw(&zjson, "body".to_string(), "details".to_string(), "destination".to_string()) {
                                                Some(key) => format!("zbus/metric/{}/{}{}", &c.protocol_version, &c.platform_name, key.as_str().unwrap()),
                                                None => continue,
                                            };
                                        }
                                        Err(err) => {
                                            log::error!("Error converting JSON: {:?}", err);
                                        }
                                    }
                                    log::debug!("End of JSON");
                                }
                                log::debug!("End of JSON series.");
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
