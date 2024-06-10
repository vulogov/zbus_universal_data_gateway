extern crate log;
use voca_rs::*;
use crate::rhai_lib::string::Text;

impl Text {
    pub fn includes(&mut self, t: String) -> bool {
        query::includes(&self.raw(), &t, 0)
    }
}
