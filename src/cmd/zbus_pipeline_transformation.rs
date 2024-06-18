extern crate log;
use crate::cmd;
use crate::stdlib;


pub fn processor(c: &cmd::Cli, pipeline: &cmd::Pipeline)  {
    log::debug!("zbus_gateway_processor_filter::run() reached");

    let script = match stdlib::zio::read_file(pipeline.script.clone().unwrap()) {
        Some(script) => script,
        None => {
            log::error!("Can not get the RHAI script");
            return;
        }
    };

    cmd::zbus_thread_transformation::processor(c, script, pipeline.telemetry_monitor_elapsed, false, pipeline.analysis);

}
