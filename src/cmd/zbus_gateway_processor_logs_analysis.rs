extern crate log;
use crate::cmd;

pub fn processor(c: &cmd::Cli, gateway: &cmd::Gateway)  {
    log::debug!("zbus_gateway_processor_logs_analysis::run() reached");

    cmd::zbus_thread_logs_analysis::processor(c, gateway.telemetry_monitor_elapsed, gateway.group.none);
}
