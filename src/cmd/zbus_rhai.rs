extern crate log;
use crate::cmd;
use rhai::{Engine, Scope, Dynamic, CallFnOptions, EvalAltResult, Map};
use rhai::packages::Package;
use rhai_sci::SciPackage;
use rhai_rand::RandomPackage;
use rhai_fs::FilesystemPackage;
use rhai_url::UrlPackage;
use rhai_ml::MLPackage;

pub fn eval_rhai_fn(script: String, c: &cmd::Cli, fun: String, v: Map) -> Result<Dynamic, Box<EvalAltResult>>  {
    log::trace!("Compiling ZBUS script and evaluating function {}() len()={}", &fun, &script.len());
    let mut engine = Engine::new();
    engine.register_global_module(SciPackage::new().as_shared_module());
    engine.register_global_module(RandomPackage::new().as_shared_module());
    engine.register_global_module(FilesystemPackage::new().as_shared_module());
    engine.register_global_module(UrlPackage::new().as_shared_module());
    engine.register_global_module(MLPackage::new().as_shared_module());
    let mut scope = Scope::new();
    let mut value: Dynamic = Dynamic::UNIT.into();

    scope.push("ZBUS_PROTOCOL_VERSION", Dynamic::from(c.protocol_version.clone()))
         .push("ZBUS_PLATFORM_NAME", Dynamic::from(c.platform_name.clone()))
         .push("ZBUS_SOURCE", Dynamic::from(c.source.clone()));

    initscope(&mut scope);
    initlib(&mut engine, c);

    let ast = match engine.compile(script) {
        Ok(ast) => ast,
        Err(err) => {
            drop(scope);
            drop(engine);
            let e = format!("Script compilation error: {}", &err);
            log::error!("{}", &e);
            return Err(e.into());
        }
    };
    let options = CallFnOptions::new()
                .eval_ast(false)
                .rewind_scope(false)
                .bind_this_ptr(&mut value);
    engine.call_fn_with_options(options, &mut scope, &ast, fun, (v,))?;
    drop(scope);
    drop(engine);
    log::debug!("ZB-script engine is no more");
    Ok(value.clone())
}

pub fn initscope(_scope: &mut Scope) {
    log::debug!("Initializing ZBUS RHAI scope");

}

pub fn initlib(_engine: &mut Engine, _c: &cmd::Cli)  {
    log::debug!("Initializing ZBUS RHAI library");

}

pub fn run(c: &cmd::Cli, gateway: &cmd::Gateway)  {
    log::debug!("zbus_rhai::run() reached");
    match &gateway.script {
        Some(f) => log::debug!("Processing will be scripted from {:?}", f),
        None => log::debug!("Processing will not be scripted"),
    }
}
