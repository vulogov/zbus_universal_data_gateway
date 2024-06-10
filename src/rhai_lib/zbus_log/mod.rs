extern crate log;
use rhai::{Engine};
use rhai::plugin::*;

#[export_module]
pub mod log_module {
    pub fn dbg(msg: &str) {
        log::debug!("{}", msg);
    }
    pub fn info(msg: &str) {
        log::info!("{}", msg);
    }
    pub fn warning(msg: &str) {
        log::warn!("{}", msg);
    }
    pub fn error(msg: &str) {
        log::error!("{}", msg);
    }
}

pub fn init(engine: &mut Engine) {
    log::debug!("Running STDLIB::log init");
    let module = exported_module!(log_module);
    engine.register_static_module("log", module.into());
}
