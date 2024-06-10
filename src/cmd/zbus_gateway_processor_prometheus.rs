extern crate log;
use crate::cmd;
use crate::stdlib;
use etime::Etime;
use nanoid::nanoid;
use prometheus_parse;
use serde_json::{json};


pub fn processor(c: &cmd::Cli, gateway: &cmd::Gateway)  {
    log::trace!("zbus_gateway_processor_prometheus::run() reached");
    let c = c.clone();
    let gateway = gateway.clone();
    match stdlib::threads::THREADS.lock() {
        Ok(t) => {
            t.execute(move ||
            {
                log::debug!("PROCESSOR PROMETHEUS thread has been started");
                loop {
                    if ! stdlib::channel::pipe_is_empty_raw("in".to_string()) {
                        match stdlib::channel::pipe_pull("in".to_string()) {
                            Ok(data) => {
                                log::debug!("Received {} bytes by processor", &data.len());
                                let mut e = Etime::new();
                                e.tic();
                                let lines: Vec<_> = data.lines().map(|s| Ok(s.to_owned())).collect();
                                let m = prometheus_parse::Scrape::parse(lines.into_iter());
                                match m {
                                    Ok(metrics) => {
                                        for s in &metrics.samples {
                                            let itemkey = &s.metric;
                                            let desc = match metrics.docs.get(&itemkey.clone()) {
                                                Some(desc) => String::from(desc),
                                                None => String::from("N/A"),
                                            };
                                            let value = match s.value {
                                                prometheus_parse::Value::Counter(value) |
                                                prometheus_parse::Value::Untyped(value) |
                                                prometheus_parse::Value::Gauge(value) =>
                                                    json!(value.clone()),
                                                _ => continue,
                                            };
                                            let timestamp = json!(s.timestamp.timestamp_millis());
                                            let content_type: u16 = if value.is_f64() {
                                                0
                                            } else if value.is_u64() {
                                                3
                                            } else if value.is_string() {
                                                4
                                            } else {
                                                10
                                            };
                                            let out = json!({
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
                                                        "origin":       c.source.clone(),
                                                        "destination":  format!("zbus/prometheus_metric/{}/{}/{}", &c.protocol_version, &c.platform_name, itemkey.clone()),
                                                        "properties":   {
                                                            "timestamp":        timestamp,
                                                            "description":      desc,
                                                            "prometheus_key":   itemkey.clone(),
                                                        },
                                                        "details":  {
                                                            "detailType":   "",
                                                            "contentType":  content_type,
                                                            "data":         value,
                                                        }
                                                    }
                                                },
                                                "id": nanoid!(),
                                            });
                                            if ! gateway.group.none {
                                                match &gateway.script {
                                                    Some(_) => {
                                                        stdlib::channel::pipe_push("filter".to_string(), out.to_string());
                                                    }
                                                    None => {
                                                        if gateway.analysis {
                                                            log::debug!("Pushing {} for analysis", &itemkey);
                                                            stdlib::channel::pipe_push("analysis".to_string(), out.to_string());
                                                        } else {
                                                            log::debug!("Pushing {} for output", &itemkey);
                                                            stdlib::channel::pipe_push("out".to_string(), out.to_string());
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                    Err(err) => {
                                        log::error!("Error parsing prometheus responce: {}", err);
                                    }
                                }
                                let elapsed = e.toc().as_secs_f32();
                                log::debug!("Elapsed time for processing: {} seconds", elapsed);
                                if gateway.telemetry_monitor_elapsed {
                                    let data = cmd::zbus_json::generate_json_telemetry(&c, "/zbus/udg/elapsed".to_string(), "Elapsed time for JSON batch processing".to_string(), 3, json!(elapsed));
                                    if ! gateway.group.none {
                                        stdlib::channel::pipe_push("out".to_string(), data.to_string());
                                    }
                                }
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
