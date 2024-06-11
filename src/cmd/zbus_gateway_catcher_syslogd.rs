extern crate log;
use crate::cmd;
use crate::stdlib;
use mio::net::UdpSocket;
use mio::{Events, Poll, PollOpt, Ready, Token};
use socket2::{Domain, Protocol, Socket, Type};
use std::io::{Error, ErrorKind};
use std::net::{Ipv4Addr, SocketAddr};
use crate::stdlib::syslog;
use serde_json::{json};
use nanoid::nanoid;

const SERVER4: Token = Token(0);

fn receive(sock: &UdpSocket, buf: &mut [u8], c: &cmd::Cli, gateway: &cmd::Gateway) -> Result<(), Error> {
    loop {
        let (len, from) = match sock.recv_from(buf) {
            Ok((len, from)) => (len, from),
            Err(e) => {
                if e.kind() == ErrorKind::WouldBlock || e.kind() == ErrorKind::Interrupted {
                    return Ok(());
                } else {
                    return Err(e);
                }
            }
        };

        if let Some(msg) = syslog::parse(from, len, buf) {
            let message = match msg.msg {
                Some(message) => message,
                None => "N/A".to_string(),
            };
            let timestamp = match msg.timestamp {
                Some(timestamp) => timestamp.timestamp_nanos_opt().unwrap(),
                None => 7,
            };
            let hostname = match msg.hostname {
                Some(hostname) => hostname,
                None => c.source.clone(),
            };
            let appname = match msg.appname {
                Some(appname) => appname,
                None => "unknown".to_string(),
            };
            let procid = match msg.procid {
                Some(procid) => procid,
                None => "unknown".to_string(),
            };
            let msgid = match msg.msgid {
                Some(msgid) => msgid,
                None => nanoid!(),
            };

            let data = json!({
                "headers": {
                    "messageType":      "telemetry",
                    "route":            c.route.clone(),
                    "streamName":       c.platform_name.clone(),
                    "cultureCode":      null,
                    "version":          c.protocol_version.clone(),
                    "encryptionAlgorithm":      null,
                    "compressionAlgorithm":     null,
                },
                "body": {
                    "details": {
                        "origin":       hostname,
                        "destination":  gateway.syslogd_key.clone(),
                        "properties":   {
                            "timestamp":        timestamp,
                            "zabbix_item":      format!("log[{}]", &gateway.syslog_file_name), 
                            "syslog_facility":  msg.facility,
                            "syslog_severity":  msg.severity,
                            "syslog_version":   msg.version,
                            "syslog_appname":   appname,
                            "syslog_procid":    procid,
                        },
                        "details":  {
                            "detailType":   "",
                            "contentType":  2,
                            "data":         message.clone(),
                        }
                    }
                },
                "id": msgid,
            });
            stdlib::channel::pipe_push("in".to_string(), data.to_string());
        } else {
            match std::str::from_utf8(buf) {
                Ok(s) => log::error!("SYSLOGD error parsing: {}", s),
                Err(e) => log::error!("SYSLOGD received message not parseable and not UTF-8: {}", e),
            }
        }
    }
}

pub fn catcher(c: &cmd::Cli, gateway: &cmd::Gateway)  {
    log::trace!("zbus_gateway_catcher_syslog::run() reached");
    let gateway = gateway.clone();
    let c = c.clone();
    match stdlib::threads::THREADS.lock() {
        Ok(t) => {
            t.execute(move ||
            {
                log::debug!("SYSLOG catcher thread has been started");
                let mut events = Events::with_capacity(gateway.syslogd_catcher_capacity.into());
                let poll = match Poll::new() {
                    Ok(poll) => poll,
                    Err(err) => {
                        log::error!("SYSLOGD error creating poll: {}", err);
                        return;
                    }
                };
                let mut buffer = [0; 4096];

                let udp4_server_s = match Socket::new(Domain::ipv4(), Type::dgram(), Some(Protocol::udp())) {
                    Ok(udp4_server) => udp4_server,
                    Err(err) => {
                        log::error!("SYSLOGD error creating server: {}", err);
                        return;
                    }
                };
                let sa4 = SocketAddr::new(Ipv4Addr::new(0, 0, 0, 0).into(), gateway.syslogd_udp_port);

                udp4_server_s.set_reuse_address(true).unwrap();
                udp4_server_s.set_reuse_port(true).unwrap();
                udp4_server_s.bind(&sa4.into()).unwrap();
                let udp4_server_mio = UdpSocket::from_socket(udp4_server_s.into_udp_socket()).unwrap();
                poll.register(
                    &udp4_server_mio,
                    SERVER4,
                    Ready::readable(),
                    PollOpt::edge(),
                ).unwrap();
                'outer: loop {
                    match poll.poll(&mut events, None) {
                        Ok(_) => {
                            for event in events.iter() {
                                match event.token() {
                                    SERVER4 => match receive(&udp4_server_mio, &mut buffer, &c, &gateway) {
                                        Ok(()) => continue,
                                        Err(err) => {
                                            log::error!("SYSLOGD IPv4 receive error: {}", err);
                                            continue 'outer;
                                        }
                                    },
                                    _ => continue 'outer,
                                }
                            }
                        }
                        Err(err) => {
                            log::error!("SYSLOGD IPv4 poll error: {}", err);
                            continue 'outer;
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
