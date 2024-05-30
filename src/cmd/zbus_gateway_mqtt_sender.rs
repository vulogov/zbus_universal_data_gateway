extern crate log;
use crate::cmd;
use crate::stdlib;
use nanoid;
use std::io::prelude::*;
use std::net::TcpStream;
use mqtt::{Encodable, Decodable};
use mqtt::packet::{QoSWithPacketIdentifier, SubscribePacket, ConnackPacket, ConnectPacket, PublishPacketRef};
use mqtt::{TopicName, TopicFilter, QualityOfService};
use mqtt::control::variable_header::ConnectReturnCode;
use serde_json::{Deserializer, Value};


pub fn sender(c: &cmd::Cli, gateway: &cmd::Gateway)  {
    log::trace!("zbus_gateway_mqtt_sender::run() reached");
    let gateway = gateway.clone();
    let c       = c.clone();

    match stdlib::threads::THREADS.lock() {
        Ok(t) => {
            t.execute(move ||
            {
                log::debug!("MQTT sender thread has been started");
                let aggregate_key = format!("{}/{}", &c.platform_name, &gateway.mqtt_aggregate_key);
                log::debug!("Published telemetry will be aggregated to: {}", &aggregate_key);
                let mut channel_filter: Vec<(TopicFilter, QualityOfService)> = Vec::new();
                let mut channels: Vec<TopicName> = Vec::new();
                channel_filter.push((TopicFilter::new(aggregate_key.clone()).unwrap(), QualityOfService::Level0));
                channels.push(TopicName::new(aggregate_key.clone()).unwrap());
                'outside: loop {
                    match TcpStream::connect(gateway.mqtt_connect.clone()) {
                        Ok(mut stream) => {
                            let mut conn = ConnectPacket::new(nanoid::nanoid!());
                            conn.set_clean_session(true);
                            let mut buf = Vec::new();
                            conn.encode(&mut buf).unwrap();
                            stream.write(&buf[..]).unwrap();
                            let connack = ConnackPacket::decode(&mut stream).unwrap();
                            if connack.connect_return_code() != ConnectReturnCode::ConnectionAccepted {
                                log::error!("MQTT returned: {:?}", connack.connect_return_code());
                                break 'outside;
                            }
                            let sub = SubscribePacket::new(10, channel_filter);
                            let mut buf = Vec::new();
                            sub.encode(&mut buf).unwrap();
                            stream.write(&buf[..]).unwrap();
                            loop {
                                if ! stdlib::channel::pipe_is_empty_raw("out".to_string()) {
                                    match stdlib::channel::pipe_pull("out".to_string()) {
                                        Ok(res) => {
                                            log::debug!("Received {} bytes by MQTT processor", &res.len());
                                            let vstream = Deserializer::from_str(&res).into_iter::<Value>();
                                            for value in vstream {
                                                match value {
                                                    Ok(zjson) => {
                                                        match serde_json::to_string(&zjson) {
                                                            Ok(payload) => {
                                                                for chan in &channels {
                                                                    let publish_packet = PublishPacketRef::new(chan, QoSWithPacketIdentifier::Level0, payload.as_bytes());
                                                                    let mut buf = Vec::new();
                                                                    publish_packet.encode(&mut buf).unwrap();
                                                                    stream.write(&buf[..]).unwrap();
                                                                }
                                                            }
                                                            Err(err) => {
                                                                log::error!("Error convert JSON to string: {}", err);
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
