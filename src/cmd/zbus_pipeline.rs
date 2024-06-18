extern crate log;
use crate::cmd;
use crate::stdlib;
use std::path::Path;
use std::str::FromStr;
use zenoh::config::{Config, ConnectConfig, ListenConfig, EndPoint, WhatAmI};
use zenoh::prelude::sync::*;

pub fn pipeline_bus_channel(_c: &cmd::Cli, pipeline: cmd::Pipeline, topic: String)  {

    match stdlib::threads::THREADS.lock() {
        Ok(t) => {
            t.execute(move ||
            {
                log::debug!("Bus->Channel thread has been started for {}", &topic);
                let mut config_in =  Config::default();

                if pipeline.zbus_disable_multicast_scout.clone() {
                    match config_in.scouting.multicast.set_enabled(Some(false)) {
                        Ok(_) => { log::debug!("Multicast discovery disabled")}
                        Err(err) => {
                            log::error!("Failure in disabling multicast discovery: {:?}", err);
                            return;
                        }
                    }
                } else {
                    log::debug!("Multicast discovery enabled");
                }
                match EndPoint::from_str(&pipeline.zbus_recv_connect) {
                    Ok(zconn) => {
                        log::debug!("ZENOH bus set to: {:?}", &zconn);
                        let _ = config_in.set_connect(ConnectConfig::new(vec![zconn]).unwrap());
                    }
                    Err(err) => {
                        log::error!("Failure in parsing connect address: {:?}", err);
                        return;
                    }
                }
                match EndPoint::from_str(&pipeline.zbus_recv_listen) {
                    Ok(zlisten) => {
                        log::debug!("ZENOH listen set to: {:?}", &zlisten);
                        let _ = config_in.set_listen(ListenConfig::new(vec![zlisten]).unwrap());
                    }
                    Err(_) => {
                        log::debug!("ZENOH listen set to default");
                    }
                }
                if pipeline.zbus_set_connect_mode {
                    log::debug!("ZENOH configured in CONNECT mode");
                    let _ = config_in.set_mode(Some(WhatAmI::Client));
                } else {
                    log::debug!("ZENOH configured in PEER mode");
                    let _ = config_in.set_mode(Some(WhatAmI::Peer));
                }
                if config_in.validate() {
                    log::debug!("ZENOH config is OK");
                } else {
                    log::error!("ZENOH config not OK");
                    return;
                }
                'outside: loop {
                    match zenoh::open(config_in.clone()).res() {
                        Ok(session) => {
                            match session.declare_subscriber(&topic)
                                    .callback_mut(move |sample| {
                                        let slices = &sample.value.payload.contiguous();
                                        match std::str::from_utf8(slices) {
                                            Ok(data) => {
                                                match serde_json::from_str::<serde_json::Value>(&data) {
                                                    Ok(zjson) => {
                                                        log::debug!("ZBUS IN pipeline received {} bytes", &data.len());
                                                        stdlib::channel::pipe_push("in".to_string(), zjson.to_string());
                                                    }
                                                    Err(err) => {
                                                        log::error!("Error while converting JSON data from ZENOH bus: {:?}", err);
                                                    }
                                                }
                                            }
                                            Err(err) => {
                                                log::error!("Error while extracting data from ZENOH bus: {:?}", err);
                                            }
                                        }
                                    })
                                    .res() {
                                Ok(_) => {
                                    let receiver = match zenoh::scout(WhatAmI::Peer, config_in.clone())
                                        .res() {
                                            Ok(receiver) => receiver,
                                            Err(err) => {
                                                log::error!("ZBUS scout had failed: {}", err);
                                                stdlib::sleep::sleep(5);
                                                continue 'outside;
                                            }
                                        };
                                    log::debug!("Running ZBUS scout to detect the health of connection for {}", &topic);
                                    while let Ok(hello) = receiver.recv() {
                                        log::trace!("ZBUS catcher received: {}", hello);
                                        std::thread::yield_now();
                                    }
                                }
                                Err(err) => {
                                    log::error!("Telemetry subscribe for key {} failed: {:?}", &topic, err);
                                    return;
                                }
                            }
                        }
                        Err(err) => {
                            log::error!("Error opening Bus() session: {:?}", err);
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

pub fn pipeline_channel_bus(_c: &cmd::Cli, pipeline: cmd::Pipeline, topic: String)  {

    let mut config_out =  Config::default();

    if pipeline.zbus_disable_multicast_scout.clone() {
        match config_out.scouting.multicast.set_enabled(Some(false)) {
            Ok(_) => { log::debug!("Multicast discovery disabled")}
            Err(err) => {
                log::error!("Failure in disabling multicast discovery: {:?}", err);
                return;
            }
        }
    } else {
        log::debug!("Multicast discovery enabled");
    }
    match EndPoint::from_str(&pipeline.zbus_send_connect) {
        Ok(zconn) => {
            log::debug!("ZENOH bus set to: {:?}", &zconn);
            let _ = config_out.set_connect(ConnectConfig::new(vec![zconn]).unwrap());
        }
        Err(err) => {
            log::error!("Failure in parsing connect address: {:?}", err);
            return;
        }
    }
    match EndPoint::from_str(&pipeline.zbus_send_listen) {
        Ok(zlisten) => {
            log::debug!("ZENOH listen set to: {:?}", &zlisten);
            let _ = config_out.set_listen(ListenConfig::new(vec![zlisten]).unwrap());
        }
        Err(_) => {
            log::debug!("ZENOH listen set to default");
        }
    }
    if pipeline.zbus_set_connect_mode {
        log::debug!("ZENOH configured in CONNECT mode");
        let _ = config_out.set_mode(Some(WhatAmI::Client));
    } else {
        log::debug!("ZENOH configured in PEER mode");
        let _ = config_out.set_mode(Some(WhatAmI::Peer));
    }
    if config_out.validate() {
        log::debug!("ZENOH config is OK");
    } else {
        log::error!("ZENOH config not OK");
        return;
    }

    match stdlib::threads::THREADS.lock() {
        Ok(t) => {
            t.execute(move ||
            {
                log::debug!("Channel->Bus thread has been started for {}", &topic);
                match zenoh::open(config_out.clone()).res() {
                    Ok(session) => {
                        loop {
                            if ! stdlib::channel::pipe_is_empty_raw("out".to_string()) {
                                while ! stdlib::channel::pipe_is_empty_raw("out".to_string()) {
                                    match stdlib::channel::pipe_pull("out".to_string()) {
                                        Ok(res) => {
                                            log::debug!("Processing data len()={} OUT to {}", &res.len(), &topic);
                                            let _ = session.put(topic.clone(), res.clone()).encoding(KnownEncoding::AppJson).res();
                                        }
                                        Err(err) => log::error!("Error getting data from ZBUS: {:?}", err),
                                    }
                                }
                            }
                            stdlib::sleep::sleep(1);
                        }
                    }
                    Err(err) => {
                        log::error!("Error opening Bus() session: {:?}", err);
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

pub fn run(c: &cmd::Cli, pipeline: &cmd::Pipeline)  {
    log::debug!("zbus_pipeline::run() reached");



    match &pipeline.script {
        Some(fname) => {
            if Path::new(&fname).exists() {
                log::debug!("Filtering and transformation enabled");
                cmd::zbus_pipeline_filter::processor(c, pipeline);
                cmd::zbus_pipeline_transformation::processor(c, pipeline);
            } else {
                log::error!("Script not found processing disabled");
                return;
            }
        }
        None => log::debug!("Filtering disabled"),
    }

    if pipeline.analysis {
        log::debug!("Analythical collection and enchancing is ON");
        cmd::zbus_pipeline_analysis::processor(c, pipeline);
    } else {
        log::debug!("Analythical collection and enchancing is OFF");
    }

    if pipeline.logs_analysis {
        log::debug!("Logs analythical enchancing is ON");
        cmd::zbus_pipeline_logs_analysis::processor(c, pipeline);
    } else {
        log::debug!("Logs analythical enchancing is OFF");
    }

    for n in &pipeline.zbus_recv_key {
        log::debug!("Launching receiver for pipeline {}", n);
        pipeline_bus_channel(c, pipeline.clone(), n.clone());
    }

    for n in &pipeline.zbus_send_key {
        log::debug!("Launching sender for pipeline {}", n);
        pipeline_channel_bus(c, pipeline.clone(), n.clone());
    }

    cmd::zbus_gateway_processor_pipeline::processor(c, pipeline);

    stdlib::threads::wait_all();
}
