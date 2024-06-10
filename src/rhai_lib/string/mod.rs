extern crate log;
use voca_rs::*;
use rhai::{Engine, Dynamic, Array};
use rhai::plugin::*;
use fancy_regex::Regex;
use crate::rhai_lib::grok;

mod zip;
mod includes;
mod eval;
pub mod fuzzy;
mod tokens;
mod tokens_text;
mod text_text;

#[derive(Debug, Clone)]
pub struct Text {
    pub t: String,
}

impl Text {
    fn new() -> Self {
        Self {
            t: String::new(),
        }
    }
    fn init() -> Text {
        Text::new()
    }
    fn init_str(t: String) -> Text {
        let mut res = Text::new();
        res.t = t.clone();
        res
    }
    pub fn raw(&mut self) -> String {
        self.t.clone()
    }
    fn lines(&mut self) -> Array {
        let mut res = Array::new();
        for l in self.t.lines() {
            let line = manipulate::trim(&manipulate::expand_tabs(&l.to_string(), 1), "");
            if line.is_empty() {
                continue;
            }
            res.push(Dynamic::from(line));
        }
        res
    }
    fn lines_grok(&mut self, mut g: grok::NRGrok, p: String) -> Array {
        let mut res = Array::new();

        for l in self.t.lines() {
            let line = manipulate::trim(&manipulate::expand_tabs(&l.to_string(), 1), "");
            res.push(Dynamic::from(g.do_match(line, p.clone())));
        }

        res
    }
    fn lines_match(&mut self, r: String) -> Array {
        let mut res = Array::new();
        match Regex::new(&r) {
            Ok(re) => {
                for l in self.t.lines() {
                    let line = manipulate::trim(&manipulate::expand_tabs(&l.to_string(), 1), "");
                    if re.is_match(&line).unwrap() {
                        res.push(Dynamic::from(line));
                    }
                }
            }
            Err(err) => {
                log::error!("Regex creation failed: {}", err);
            }
        }
        res
    }
}

#[export_module]
pub mod string_module {
    pub fn trim(s: &str) -> String {
    	s.trim().into()
    }

    pub fn lowercase(s: &str) -> String {
    	s.to_lowercase()
    }

    pub fn uppercase(s: &str) -> String {
    	s.to_uppercase()
    }

    pub fn starts_with(a: &str, b: &str) -> bool {
    	a.starts_with(b)
    }

    pub fn ends_with(a: &str, b: &str) -> bool {
    	a.ends_with(b)
    }
    pub fn includes(a: &str, b: &str) -> bool {
    	query::includes(a,b,0)
    }
    pub fn matches(a: &str, b: &str) -> bool {
    	query::matches(a,b,0)
    }
}

pub fn init(engine: &mut Engine) {
    log::debug!("Running STDLIB::str init");
    let mut module = exported_module!(string_module);
    module.set_native_fn("zip", zip::lines_zip);
    module.set_native_fn("expr", eval::str_eval);
    module.set_native_fn("expr", eval::txt_eval);
    let mut fuzzy_module = Module::new();
    fuzzy_module.set_native_fn("Match", fuzzy::str_match);
    fuzzy_module.set_native_fn("Levenshtein", fuzzy::str_match_levenshtein);
    fuzzy_module.set_native_fn("Damerau", fuzzy::str_match_damerau);
    module.set_sub_module("fuzzy", fuzzy_module);

    let mut token_module = Module::new();
    token_module.set_native_fn("tokenize", tokens::str_tokenize);
    token_module.set_native_fn("tokenize", tokens::str_tokenize_text);

    module.set_sub_module("tokenize", token_module);

    engine.register_static_module("str", module.into());


    engine.register_type::<Text>()
          .register_fn("Text", Text::init)
          .register_fn("Text", Text::init_str)
          .register_fn("raw", Text::raw)
          .register_fn("lines", Text::lines)
          .register_fn("lines", Text::lines_match)
          .register_fn("lines", Text::lines_grok)
          .register_fn("includes", Text::includes)
          .register_fn("tokenize", Text::tokenize)
          .register_fn("text", Text::text)
          .register_fn("words", Text::words)
          .register_fn("to_string", |x: &mut Text| format!("{}", x.t) );

}
