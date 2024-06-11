extern crate log;
use crate::cmd;
use crate::stdlib;
use rhai::{Array};

pub fn catcher(c: &cmd::Cli, gateway: &cmd::Gateway)  {
    log::trace!("zbus_gateway_catcher_rhai::run() reached");
    let gateway = gateway.clone();
    let c = c.clone();
    match stdlib::threads::THREADS.lock() {
        Ok(t) => {
            t.execute(move ||
            {
                log::debug!("RHAI catcher thread has been started");

                let script = match stdlib::zio::read_file(gateway.script.clone().unwrap()) {
                    Some(script) => script,
                    None => {
                        log::error!("Can not get the RHAI script");
                        return;
                    }
                };
                let (engine, mut scope, ast) = match cmd::zbus_rhai::make_rhai_env_and_ast(script, &c) {
                    Ok((engine, scope, ast)) => (engine, scope, ast),
                    Err(err) => {
                        log::error!("Error creating RHAI instance: {}", err);
                        return;
                    }
                };

                loop {
                    match engine.call_fn::<Array>(&mut scope, &ast, "generator", ()) {
                        Ok(generator_result) => {
                            for val in generator_result.iter() {
                                match serde_json::to_string(&val.clone()) {
                                    Ok(data) => {
                                        stdlib::channel::pipe_push("in".to_string(), data);
                                    }
                                    Err(err) => {
                                        log::error!("Error converting RHAI generation result to JSON: {}", err);
                                    }
                                }
                            }
                        }
                        Err(err) => {
                            log::error!("Error in RHAI generaor: {}", err);
                        }
                    }
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
