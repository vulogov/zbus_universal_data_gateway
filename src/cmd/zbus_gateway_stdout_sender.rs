extern crate log;
use crate::cmd;
use crate::stdlib;

pub fn sender(c: &cmd::Cli, gateway: &cmd::Gateway)  {
    log::trace!("zbus_gateway_stdout_sender::run() reached");
    match stdlib::threads::THREADS.lock() {
        Ok(t) => {
            t.execute(move ||
            {
                log::debug!("STDOUT sender thread has been started");
                loop {
                    if ! stdlib::channel::pipe_is_empty_raw("out".to_string()) {
                        match stdlib::channel::pipe_pull("out".to_string()) {
                            Ok(res) => {
                                println!("{}", &res);
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
