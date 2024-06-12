extern crate log;
use std::collections::VecDeque;
use serde_json::{Value};

#[derive(Debug, Clone)]
pub struct ValueSampler {
    d: VecDeque<Value>,
}

impl ValueSampler {
    fn new() -> Self {
        Self {
            d: VecDeque::with_capacity(128),
        }
    }
    pub fn init() -> ValueSampler {
        let res = ValueSampler::new();
        res
    }
    pub fn len(self: &mut ValueSampler) -> usize {
        self.d.len()
    }
    pub fn data(self: &ValueSampler) -> Vec::<Value> {
        let mut out: Vec<Value> = Vec::new();
        for v in self.d.iter().collect::<Vec<_>>() {
            out.push(v.clone());
        }
        out
    }
    pub fn last(self: &ValueSampler) -> Option<Value> {
        if self.d.len() > 0 {
            match self.d.back() {
                Some(val) => return Some(val.clone()),
                None => return None,
            }
        }
        None
    }
    pub fn set(self: &mut ValueSampler, v: Value) {
        if self.d.len() == self.d.capacity() {
            let _ = self.d.pop_front();
        }
        let _ = self.d.push_back(v.clone());
    }
}
