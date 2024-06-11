extern crate log;
use crate::rhai_lib::string::Text;
use charabia::{Tokenize, TokenKind};
use rhai::{Array, Dynamic, Map};

impl Text {
    pub fn text(&mut self) -> Dynamic {
        let orig = self.t.as_str();
        let mut res = Array::new();
        let tokens = orig.tokenize();
        for t in tokens {
            let mut elem = Map::new();
            elem.insert("value".into(), t.lemma().into());
            match t.kind() {
                TokenKind::Word => {
                    elem.insert("word".into(), Dynamic::from(true));
                    elem.insert("separator".into(), Dynamic::from(false));
                    elem.insert("unknown".into(), Dynamic::from(false));
                }
                TokenKind::Separator(_) => {
                    elem.insert("word".into(), Dynamic::from(false));
                    elem.insert("separator".into(), Dynamic::from(true));
                    elem.insert("unknown".into(), Dynamic::from(false));
                }
                _ => {
                    elem.insert("word".into(), Dynamic::from(false));
                    elem.insert("separator".into(), Dynamic::from(false));
                    elem.insert("unknown".into(), Dynamic::from(true));
                }
            }
            res.push(Dynamic::from(elem));
        }
        return Dynamic::from(res);
    }
    pub fn words(&mut self) -> Dynamic {
        let orig = self.t.as_str();
        let mut res = Array::new();
        let tokens = orig.tokenize();
        for t in tokens {
            match &t.kind() {
                TokenKind::Word => {
                    let word = t.lemma().into();
                    res.push(word);
                }
                _ => {
                    continue;
                }
            }
        }
        return Dynamic::from(res);
    }
}
