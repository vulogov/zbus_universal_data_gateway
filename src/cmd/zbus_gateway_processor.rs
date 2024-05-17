extern crate log;
use crate::cmd;
use crate::stdlib;

pub fn processor(c: &cmd::Cli, gateway: &cmd::Gateway)  {
    log::trace!("zbus_gateway_processor::run() reached");
    match stdlib::threads::THREADS.lock() {
        Ok(t) => {
            t.execute(move ||
            {
                log::debug!("PROCESSOR thread has been started");
                loop {
                    if ! stdlib::channel::pipe_is_empty_raw("in".to_string()) {
                        match stdlib::channel::pipe_pull("in".to_string()) {
                            Ok(res) => {
                                stdlib::channel::pipe_push("out".to_string(), res);
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
