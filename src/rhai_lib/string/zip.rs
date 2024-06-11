extern crate log;
use voca_rs::*;
use crate::rhai_lib::string::Text;
use rhai::{Dynamic, Array, FnPtr, NativeCallContext, EvalAltResult};

pub fn lines_zip(context: NativeCallContext, t: &mut Text, f: FnPtr) -> Result<Vec<rhai::Dynamic>, Box<EvalAltResult>> {
    let mut res = Array::new();
    for l in t.lines() {
        let line = manipulate::trim(&manipulate::expand_tabs(&l.to_string(), 1), "");
        let r: Result<Dynamic, Box<EvalAltResult>> = f.call_within_context(&context, (line,));
        match r {
            Ok(val) => res.push(val),
            Err(_) => continue,
        }
    }
    return Result::Ok(res);
}
