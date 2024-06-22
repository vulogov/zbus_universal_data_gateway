extern crate log;
use crate::cmd;
use crate::stdlib;

pub fn processor(c: &cmd::Cli, alerts: &cmd::Alerts)  {
    log::debug!("zbus_gateway_processor_filter::run() reached");
    let script = match stdlib::zio::read_file(alerts.script.clone().unwrap()) {
        Some(script) => script,
        None => {
            log::error!("Can not get the RHAI script");
            return;
        }
    };
    cmd::zbus_thread_filter::processor(c, script, false, false);
}
