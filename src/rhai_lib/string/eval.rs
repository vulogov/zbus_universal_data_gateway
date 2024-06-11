extern crate log;
use crate::rhai_lib::string::Text;
use rhai::{Dynamic, NativeCallContext, EvalAltResult};

pub fn txt_eval(context: NativeCallContext, t: &mut Text) -> Result<rhai::Dynamic, Box<EvalAltResult>> {
    context.engine().eval_expression::<Dynamic>(&t.raw())
}

pub fn str_eval(context: NativeCallContext, t: String) -> Result<rhai::Dynamic, Box<EvalAltResult>> {
    context.engine().eval_expression::<Dynamic>(&t)
}
