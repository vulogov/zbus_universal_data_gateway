extern crate log;
use crate::cmd;
use crate::stdlib;
use nats;
use std::str::from_utf8;

pub fn catcher(c: &cmd::Cli, gateway: &cmd::Gateway)  {
    log::trace!("zbus_gateway_catcher_nats::run() reached");
    let subscribe_key = format!("zbus/metric/{}/{}/{}", &c.protocol_version, &c.platform_name, &gateway.nats_subscribe_key);
    let gateway = gateway.clone();
    match stdlib::threads::THREADS.lock() {
        Ok(t) => {
            t.execute(move ||
            {
                log::debug!("NATS catcher thread has been started");
                'outside: loop {
                    match nats::connect(gateway.nats_connect.clone()) {
                        Ok(session) => {
                            loop {
                                match session.subscribe(&subscribe_key) {
                                    Ok(sub) => {
                                        log::debug!("Subscribed to NATS: {}:{}", &gateway.nats_connect, &subscribe_key);
                                        loop {
                                            match sub.next() {
                                                Some(msg) => {
                                                    match from_utf8(&msg.data) {
                                                        Ok(content) => {
                                                            log::debug!("NATS catcher received {} bytes", &content.len());
                                                            stdlib::channel::pipe_push("in".to_string(), content.to_string());
                                                        }
                                                        Err(_) => continue,
                                                    }
                                                }
                                                None => break,
                                            }
                                        }
                                        let _ = sub.unsubscribe();
                                    }
                                    Err(err) => {
                                        log::error!("Error subscribing on NATS channel: {}", err);
                                        continue 'outside;
                                    }
                                }
                            }
                        }
                        Err(err) => {
                            log::error!("Error connecting to NATS: {}", err);
                            return;
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
