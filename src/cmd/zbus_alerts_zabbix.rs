extern crate log;
use crate::cmd;
use crate::stdlib;
use tiny_http::{Method};
use std::sync::Arc;

pub fn catcher(_c: &cmd::Cli, alerts: &cmd::Alerts)  {
    log::trace!("zbus_alerts_zabbix::run() reached");

    match tiny_http::Server::http(alerts.listen.clone()) {
        Ok(server) => {
            let server = Arc::new(server);
            for i in 0..alerts.threads {
                log::debug!("Starting zabbix alerts catching thread #{}", i);
                let server = server.clone();
                match stdlib::threads::THREADS.lock() {
                    Ok(t) => {
                        t.execute(move ||
                        {
                            loop {
                                match server.recv() {
                                    Ok(mut request) => {
                                        if request.body_length() > Some(0) {
                                            let mut content = String::new();
                                            match request.as_reader().read_to_string(&mut content) {
                                                Ok(_) => {
                                                    match request.method() {
                                                        Method::Post => {
                                                            stdlib::channel::pipe_push("in".to_string(), content);
                                                        }
                                                        _ => {
                                                            let response = tiny_http::Response::empty(422);
                                                            let _ = request.respond(response);
                                                            continue;
                                                        }
                                                    }
                                                }
                                                Err(err) => {
                                                    log::error!("Error getting request body: {:?}", err);
                                                    let response = tiny_http::Response::empty(422);
                                                    let _ = request.respond(response);
                                                    continue;
                                                }
                                            }
                                        }
                                        let response = tiny_http::Response::empty(200);
                                        let _ = request.respond(response);
                                    }
                                    Err(err) => {
                                        log::error!("Error receiving request: {:?}", err);
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
        }
        Err(err) => {
            log::error!("Error creating catcher server: {:?}", err);
        }
    }
}
