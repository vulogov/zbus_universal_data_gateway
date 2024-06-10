extern crate log;
use crate::cmd;
use crate::stdlib;


pub fn catcher(_c: &cmd::Cli, gateway: &cmd::Gateway)  {
    log::trace!("zbus_gateway_catcher_rhai::run() reached");
    let gateway = gateway.clone();

    match stdlib::threads::THREADS.lock() {
        Ok(t) => {
            t.execute(move ||
            {
                log::debug!("RHAI catcher thread has been started");
                loop {
                    stdlib::sleep::sleep(gateway.rhai_catcher_run_every.into());
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
