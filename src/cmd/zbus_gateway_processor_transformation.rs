extern crate log;
use crate::cmd;
use crate::stdlib;


pub fn processor(c: &cmd::Cli, gateway: &cmd::Gateway)  {
    log::debug!("zbus_gateway_processor_filter::run() reached");

    let script = match stdlib::zio::read_file(gateway.script.clone().unwrap()) {
        Some(script) => script,
        None => {
            log::error!("Can not get the RHAI script");
            return;
        }
    };

    cmd::zbus_thread_transformation::processor(c, script, gateway.telemetry_monitor_elapsed, gateway.group.none, gateway.analysis);

}
