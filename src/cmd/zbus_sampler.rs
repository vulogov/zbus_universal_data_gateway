extern crate log;
use std::collections::VecDeque;
use crate::stdlib::tsf::TSF;
use crate::stdlib::forecast_oscillator::FOSC;
use crate::stdlib::traits::Indicator;
use statrs::statistics::Statistics;
use markov_chain::Chain;
use decorum::{R64};

#[derive(Debug, Clone)]
pub struct Sampler {
    d: VecDeque<f64>,
    n: i64,
    q: f64,
    tsf: TSF,
    fosc: FOSC,
    tsf_next: f64,
    fosc_next: f64,
}

impl Sampler {
    fn new() -> Self {
        Self {
            d: VecDeque::with_capacity(128),
            tsf: TSF::new(8),
            fosc: FOSC::new(8),
            tsf_next: 0.0 as f64,
            fosc_next: 0.0 as f64,
            n: 0 as i64,
            q: 0.01 as f64,
        }
    }
    pub fn init() -> Sampler {
        let res = Sampler::new();
        res
    }
    pub fn len(self: &mut Sampler) -> usize {
        self.d.len()
    }
    pub fn data(self: &mut Sampler) -> Vec::<f64> {
        let mut out: Vec<f64> = Vec::new();
        for v in self.d.iter().collect::<Vec<_>>() {
            out.push(v.clone());
        }
        out
    }
    pub fn tsf_next(self: &mut Sampler) -> f64 {
        self.tsf_next
    }
    pub fn oscillator(self: &mut Sampler) -> f64 {
        self.fosc_next
    }
    pub fn set(self: &mut Sampler, v: f64) {
        if self.d.len() == self.d.capacity() {
            let _ = self.d.pop_front();
        }
        match self.tsf.next(v.clone()) {
            Some(next_val) => {
                self.tsf_next = next_val;
            }
            None => self.tsf_next = v.clone(),
        }
        match self.fosc.next(v.clone()) {
            Some(next_val) => {
                self.fosc_next = next_val;
            }
            None => self.fosc_next = v.clone(),
        }
        let _ = self.d.push_back(v);
    }
    pub fn n_get(&mut self) -> i64 {
        self.n
    }

    pub fn n_set(&mut self, v: i64) {
        self.n = v;
    }

    pub fn q_set(&mut self, v: f64) {
        self.q = v;
    }

    pub fn q_get(&mut self) -> f64 {
        self.q
    }

    pub fn set_consistent(&mut self, v: f64) -> bool {
        self.set(v);
        if (self.len() as i64 + 1) >= self.n {
            let vals = self.data();
            let s_dev = vals.std_dev();
            if s_dev.is_nan() {
                return false;
            }
            if s_dev <= self.q_get() {
                return true;
            }
        }
        false
    }
    pub fn markov(&mut self) -> Vec<f64> {
        let source = self.data();
        let mut dst: Vec<R64> = Vec::new();
        for v in source {
            dst.push(v.into());
        }
        let mut palanteer = Chain::<R64>::new(8);
        palanteer.train(dst);
        let res = palanteer.generate_limit(8);
        let mut out: Vec<f64> = Vec::new();
        for i in res {
            out.push(f64::from(i));
        }
        out
    }
    pub fn consistency(&mut self) -> f64 {
        let vals = self.data();
        let s_dev = vals.std_dev();
        return s_dev;
    }
}
