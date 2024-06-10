extern crate log;
use rhai::{Engine, Map, Identifier, Dynamic};
use grok::Grok;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct NRGrok {
    g: HashMap<String, String>,
}

impl NRGrok {
    fn new() -> Self {
        Self {
            g: HashMap::new(),
        }
    }
    fn init() -> NRGrok {
        NRGrok::new()
    }
    fn get_field(&mut self, index: String) -> String {
        if self.g.contains_key(&index) {
            return self.g.get(&index).unwrap().clone();
        }
        return "".to_string();
    }
    fn set_field(&mut self, index: String, value: String) {
        self.g.insert(index, value);
    }
    pub fn do_match(&mut self, s: String, p: String) -> Map {
        let mut grok = Grok::with_default_patterns();
        for (k, v) in  &self.g {
            grok.add_pattern(k, v);
        }
        let mut res = Map::new();
        match grok.compile(&p, false) {
            Ok(patt) => {
                match patt.match_against(&s) {
                    Some(m) => {
                        for (k, v) in &m {
                            if ! &v.is_empty() {
                                let key = Identifier::from(k);
                                let val = String::from(v);
                                res.insert(key, Dynamic::from(val));
                            }
                        }
                    },
                    None    => {},
                }
            }
            Err(err) => {
                log::error!("Error compile GROK pattern: {}", err);
            }
        }
        res
    }
    fn is_match(&mut self, s: String, p: String) -> bool {
        let mut grok = Grok::with_default_patterns();
        for (k, v) in  &self.g {
            grok.add_pattern(k, v);
        }
        match grok.compile(&p, false) {
            Ok(patt) => {
                match patt.match_against(&s) {
                    Some(m) => {
                        for (_, v) in &m {
                            if v.is_empty() {
                                return false;
                            }
                        }
                        true
                    },
                    None    => false,
                }
            }
            Err(err) => {
                log::error!("Error compile GROK pattern: {}", err);
                false
            }
        }
    }
}

pub fn init(engine: &mut Engine) {
    log::debug!("Running STDLIB::grok init");

    engine.register_type::<NRGrok>()
          .register_fn("Grok", NRGrok::init)
          .register_fn("is_match", NRGrok::is_match)
          .register_fn("do_match", NRGrok::do_match)
          .register_indexer_set(NRGrok::set_field)
          .register_indexer_get(NRGrok::get_field)
          .register_fn("to_string", |x: &mut NRGrok| format!("{:?}", x.g) );

}
