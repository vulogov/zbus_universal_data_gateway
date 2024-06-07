extern crate log;

use crate::stdlib::tsf::TSF;
use crate::stdlib::traits::Indicator;

#[derive(Debug,Clone)]
pub struct FOSC {
    prev_tsf: Option<f64>,
    tsf: TSF,
}


impl FOSC {
    pub fn new(period: u32) -> FOSC {
        Self {
            prev_tsf: None,
            tsf: TSF::new(period),
        }
    }
}

impl Indicator<f64, Option<f64>> for FOSC {
    fn next(&mut self, input: f64) -> Option<f64> {
        let res = match self.prev_tsf {
            None => None,
            Some(prev) => {
                let fosc = 100.0 * ((input - prev) / input);
                Some(fosc)
            }
        };
        self.prev_tsf = self.tsf.next(input);
        res
    }

    fn reset(&mut self) {
        self.prev_tsf = None;
        self.tsf.reset();
    }
}
