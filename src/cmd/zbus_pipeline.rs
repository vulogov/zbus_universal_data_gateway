extern crate log;
use crate::cmd;
use crate::stdlib;
use std::path::Path;
use std::str::FromStr;
use zenoh::config::{Config, ConnectConfig, ListenConfig, EndPoint, WhatAmI};
use zenoh::prelude::sync::*;

pub fn pipeline_bus_channel(_c: &cmd::Cli, topic: String, zc: Config)  {

    match stdlib::threads::THREADS.lock() {
        Ok(t) => {
            t.execute(move ||
            {
                log::debug!("Bus->Channel thread has been started for {}", &topic);
                'outside: loop {
                    match zenoh::open(zc.clone()).res() {
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
                                    let receiver = match zenoh::scout(WhatAmI::Peer, zc.clone())
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

// pub fn pipeline_channel_bus(c: cmd::Cli, topic: String, zc: Config)  {
//
//     match stdlib::threads::THREADS.lock() {
//         Ok(t) => {
//             t.execute(move ||
//             {
//                 log::debug!("Channel->Bus thread has been started for {}", &bus_key);
//                 match zenoh::open(zc.clone()).res() {
//                     Ok(session) => {
//                         let pipeline_name = format!("zbus/pipeline/{}/{}", &c.protocol_version, &bus_key);
//                         loop {
//                             if ! zbus_lib::bus::channel::pipe_is_empty_raw(c_name.clone()) {
//                                 log::debug!("Processing data in {} channel to {}", &c_name, &pipeline_name);
//                                 while ! zbus_lib::bus::channel::pipe_is_empty_raw(c_name.clone()) {
//                                     match zbus_lib::bus::channel::pipe_pull_raw(c_name.clone()) {
//                                         Ok(res) => {
//                                             let _ = session.put(pipeline_name.clone(), res.clone()).encoding(KnownEncoding::AppJson).res();
//                                         }
//                                         Err(err) => log::error!("Error getting data from ZBUS: {:?}", err),
//                                     }
//                                 }
//                             }
//                             zbus_lib::system::system_module::sleep(1);
//                         }
//                     }
//                     Err(err) => {
//                         log::error!("Error opening Bus() session: {:?}", err);
//                     }
//                 }
//             });
//             drop(t);
//         }
//         Err(err) => {
//             log::error!("Error accessing Thread Manager: {:?}", err);
//             return;
//         }
//     }
//
// }

pub fn run(c: &cmd::Cli, pipeline: &cmd::Pipeline)  {
    log::debug!("zbus_pipeline::run() reached");

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

    for n in &pipeline.zbus_recv_key {
        log::debug!("Launching processor for pipeline {}", n);
        pipeline_bus_channel(c, n.clone(), config_in.clone());
    }

    stdlib::threads::wait_all();
}
