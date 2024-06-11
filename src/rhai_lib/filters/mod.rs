extern crate log;
use rhai::{Engine, Module};

pub mod interval;

pub fn init(engine: &mut Engine) {
    log::debug!("Running STDLIB::filters init");
    let mut filter_module = Module::new();
    interval::init(engine, &mut filter_module);
    engine.register_static_module("filter", filter_module.into());
}
