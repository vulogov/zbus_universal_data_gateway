extern crate log;
use std::io::prelude::*;
use std::net::{TcpStream, Shutdown};
use std::time::Duration;
use crate::cmd;
use crate::stdlib;
use serde_json::{Deserializer, Value};

pub fn sender(_c: &cmd::Cli, gateway: &cmd::Gateway)  {
    log::trace!("zbus_gateway_stdout_sender::run() reached");
    let gateway = gateway.clone();

    match stdlib::threads::THREADS.lock() {
        Ok(t) => {
            t.execute(move ||
            {
                log::debug!("STDOUT sender thread has been started");
                'connect_loop: loop {
                    match TcpStream::connect(gateway.tcp_connect.clone()) {
                        Ok(mut stream) => {
                            match stream.set_write_timeout(Some(Duration::new(gateway.tcp_timeout.into(), 0))) {
                                Ok(_) => {}
                                Err(err) => {
                                    log::error!("Error setting write timeout: {}", err);
                                }
                            }
                            loop {
                                if ! stdlib::channel::pipe_is_empty_raw("out".to_string()) {
                                    match stdlib::channel::pipe_pull("out".to_string()) {
                                        Ok(res) => {
                                            log::debug!("Received {} bytes by STDOUT processor", &res.len());
                                            let zstream = Deserializer::from_str(&res).into_iter::<Value>();
                                            for value in zstream {
                                                match value {
                                                    Ok(zjson) => {
                                                        let data = format!("{}\n", &zjson.to_string());
                                                        match stream.write(&data.into_bytes()) {
                                                            Ok(_) => {}
                                                            Err(err) => {
                                                                let _ = stream.shutdown(Shutdown::Both);
                                                                log::error!("Error in write to TCP stream: {}", err);
                                                                continue 'connect_loop;
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
                            log::error!("Error connecting to {}: {}", &gateway.tcp_connect, err);
                            stdlib::sleep::sleep(gateway.tcp_timeout.into());
                            continue 'connect_loop;
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
