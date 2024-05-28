extern crate log;
use crate::cmd;
use crate::stdlib;
use nats;
use serde_json::{Deserializer, Value};


pub fn sender(c: &cmd::Cli, gateway: &cmd::Gateway)  {
    log::trace!("zbus_gateway_nats_sender::run() reached");
    let gateway = gateway.clone();
    let c       = c.clone();
    match stdlib::threads::THREADS.lock() {
        Ok(t) => {
            t.execute(move ||
            {
                log::debug!("NATS sender thread has been started");

                let aggregate_key = format!("zbus/metric/{}/{}/{}", &c.protocol_version, &c.platform_name, &gateway.nats_aggregate_key);
                if gateway.nats_aggregate {
                    log::debug!("Published telemetry will be aggregated to: {}", &aggregate_key);
                } else {
                    log::debug!("Published telemetry will not be aggregated");
                }
                match nats::connect(gateway.nats_connect.clone()) {
                    Ok(session) => {
                        loop {
                            if ! stdlib::channel::pipe_is_empty_raw("out".to_string()) {
                                match stdlib::channel::pipe_pull("out".to_string()) {
                                    Ok(res) => {
                                        log::debug!("Received {} bytes by ZBUS processor", &res.len());
                                        let stream = Deserializer::from_str(&res).into_iter::<Value>();
                                        for value in stream {
                                            match value {
                                                Ok(zjson) => {
                                                    match serde_json::to_string(&zjson) {
                                                        Ok(payload) => {
                                                            if gateway.nats_aggregate {
                                                                match session.publish(&aggregate_key.clone(), payload.clone()) {
                                                                    Ok(_) => log::debug!("ZBX catcher->NATS: {} len()={} bytes", &aggregate_key, &payload.len()),
                                                                    Err(err) => log::error!("Error ingesting {} {:?}: {:?}", &aggregate_key, &payload, err),
                                                                }
                                                            } else {
                                                                let itemkey = match cmd::zbus_gateway_processor::zabbix_json_get_sub_subkey_raw(&zjson, "body".to_string(), "details".to_string(), "destination".to_string()) {
                                                                    Some(key) => format!("zbus/metric/{}/{}{}", &c.protocol_version, &c.platform_name, key.as_str().unwrap()),
                                                                    None => continue,
                                                                };
                                                                match session.publish(&itemkey.clone(), payload.clone()) {
                                                                    Ok(_) => log::debug!("ZBX catcher->NATS: {} len()={} bytes", &itemkey, &payload.len()),
                                                                    Err(err) => log::error!("Error ingesting {} {:?}: {:?}", &itemkey, &payload, err),
                                                                }
                                                            }
                                                        }
                                                        Err(err) => {
                                                            log::error!("Error convert JSON to string: {}", err);
                                                        }
                                                    }
                                                    // println!("{}", &zjson);
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
                        log::error!("Error connecting to the bus: {:?}", err);
                        return;
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
