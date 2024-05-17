extern crate log;
use env_logger::{Env};
use crate::cmd;


pub fn setloglevel(c: &cmd::Cli) {
    let env = Env::default().filter_or("ZBUS_LOG_LEVEL", "error");
    match c.debug {
        0 => {
            env_logger::init_from_env(env);
        }
        1 => {
            let env = Env::default()
                .filter_or("ZBUS_LOG_LEVEL", "info");
            env_logger::init_from_env(env);
        }
        2 => {
            let env = Env::default()
                .filter_or("ZBUS_LOG_LEVEL", "debug");
            env_logger::init_from_env(env);
        }
        _ => {
            let env = Env::default()
                .filter_or("ZBUS_LOG_LEVEL", "trace");
            env_logger::init_from_env(env);
        }
    }
    log::trace!("setloglevel::setloglevel() reached")
}
