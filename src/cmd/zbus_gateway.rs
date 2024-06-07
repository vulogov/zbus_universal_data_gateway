extern crate log;
use crate::cmd;
use crate::stdlib;
use std::path::Path;

pub fn run(c: &cmd::Cli, gateway: &cmd::Gateway)  {
    log::trace!("zbus_gateway::run() reached");

    if gateway.catchers.zabbix {
        cmd::zbus_gateway_processor::processor(c, gateway);
    } else if gateway.catchers.nats_catcher {
        cmd::zbus_gateway_processor_passthrough::processor(c, gateway);
    } else if gateway.catchers.zbus_catcher {
        cmd::zbus_gateway_processor_passthrough::processor(c, gateway);
    } else {
        log::error!("Catcher is not specified");
        return;
    }
    match &gateway.script {
        Some(fname) => {
            if Path::new(&fname).exists() {
                log::debug!("Filtering and transformation enabled");
                cmd::zbus_gateway_processor_filter::processor(c, gateway);
                cmd::zbus_gateway_processor_transformation::processor(c, gateway);
            } else {
                log::error!("Script not found processing disabled");
                return;
            }
        }
        None => log::debug!("Filtering disabled"),
    }

    if gateway.analysis {
        log::debug!("Analythical collection and enchancing is ON");
        cmd::zbus_gateway_processor_analysis::processor(c, gateway);
    } else {
        log::debug!("Analythical collection and enchancing is OFF");
    }

    if gateway.group.stdout {
        cmd::zbus_gateway_stdout_sender::sender(c, gateway);
    } else if gateway.group.socket {
        cmd::zbus_gateway_tcpsocket_sender::sender(c, gateway);
    } else if gateway.group.zbus {
        cmd::zbus_gateway_zbus_sender::sender(c, gateway);
    } else if gateway.group.nats {
        cmd::zbus_gateway_nats_sender::sender(c, gateway);
    } else if gateway.group.mqtt {
        cmd::zbus_gateway_mqtt_sender::sender(c, gateway);
    } else if gateway.group.statsd {
        cmd::zbus_gateway_statsd_sender::sender(c, gateway);
    } else if gateway.group.telegraf {
        cmd::zbus_gateway_telegraf_sender::sender(c, gateway);
    } else if gateway.group.clickhouse {
        cmd::zbus_gateway_clickhouse_sender::sender(c, gateway);
    } else if gateway.group.none {
        log::info!("Sender is set to NONE");
    } else {
        log::error!("Sender is not specified");
        return;
    }

    if gateway.catchers.zabbix {
        cmd::zbus_gateway_catcher_zabbix::catcher(c, gateway);
    } else if gateway.catchers.nats_catcher {
        cmd::zbus_gateway_catcher_nats::catcher(c, gateway);
    } else if gateway.catchers.zbus_catcher {
        cmd::zbus_gateway_catcher_zbus::catcher(c, gateway);
    } else {
        log::error!("Catcher is not specified");
        return;
    }

    stdlib::threads::wait_all();
}
