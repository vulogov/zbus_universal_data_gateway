extern crate log;
use crate::cmd;
use crate::stdlib;
use tiny_http::{Method};
use std::thread;
use std::sync::Arc;

pub fn catcher(_c: &cmd::Cli, gateway: &cmd::Gateway)  {
    log::trace!("zbus_gateway_catcher_zabbix::run() reached");

    match tiny_http::Server::http(gateway.listen.clone()) {
        Ok(server) => {
            let mut guards = Vec::with_capacity(gateway.threads.into());
            let server = Arc::new(server);
            for i in 0..gateway.threads {
                log::debug!("Starting zabbix catching thread #{}", i);
                let server = server.clone();
                // let gateway = gateway.clone();
                // let c = c.clone();
                // let i = i.clone();
                let guard = thread::spawn(move || {
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
                guards.push(guard);
            }
            for h in guards {
                match h.join() {
                    Ok(_) => {}
                    Err(err) => log::error!("Zabbix catcher error in joining the thread: {:?}", err),
                }
            }
        }
        Err(err) => {
            log::error!("Error creating catcher server: {:?}", err);
        }
    }
}
