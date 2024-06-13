extern crate log;
use crate::cmd;

pub fn processor(c: &cmd::Cli, pipeline: &cmd::Pipeline)  {
    log::debug!("zbus_pipeline_analysis::run() reached");

    cmd::zbus_thread_analysis::processor(c, pipeline.telemetry_monitor_elapsed, false, pipeline.anomalies_window);
}
