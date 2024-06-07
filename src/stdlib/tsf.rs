use crate::stdlib::fixedqueue::FixedQueue;
use crate::stdlib::traits::Indicator;

#[derive(Debug,Clone)]
pub struct TSF {
    x_mean: f64,
    x_var_sq: f64,
    sma: SMA,
    history: FixedQueue<f64>,
    period: u32,

}

#[derive(Debug,Clone)]
pub struct SMA {
    accumulator: f64,
    period: u32,
    history: FixedQueue<f64>,
}

impl SMA {
    pub fn new(period: u32) -> SMA {
        Self {
            accumulator: 0_f64,
            period,
            history: FixedQueue::new(period),
        }
    }

    pub fn get_total(&self) -> f64 {
        return self.accumulator;
    }
}

impl Indicator<f64, Option<f64>> for SMA {
    fn next(&mut self, input: f64) -> Option<f64> {
        if self.history.is_full() {
            let out = self.history.at(0).unwrap();
            self.history.add(input);
            self.accumulator = self.accumulator - out + input;
        } else {
            self.history.add(input);
            self.accumulator = self.accumulator + input;
        }
        if self.history.is_full() {
            return Some(self.accumulator/self.period as f64);
        }
        None
    }

    fn reset(&mut self) {
        self.history.clear();
        self.accumulator = 0_f64;
    }
}

impl TSF {
    pub fn new(period: u32) -> TSF {
        let mean = ((period as f64 * (period as f64 + 1.0)) / 2.0) / period as f64;
        Self {
            x_mean: mean,
            sma: SMA::new(period),
            history: FixedQueue::new(period),
            period,
            x_var_sq: {
                let mut sum = 0.0;
                for i in 1..(period + 1) {
                    sum += (i as f64 - mean).powf(2.0);
                }
                sum
            },
        }
    }
}


impl Indicator<f64, Option<f64>> for TSF {
    fn next(&mut self, input: f64) -> Option<f64> {
        let sma = self.sma.next(input);
        self.history.add(input);
        match sma {
            None => None,
            Some(sm) => {
                let mut beta = 0.0;
                for i in 1..(self.period + 1) {
                    beta += ((i as f64 - self.x_mean) * (self.history.at((i - 1) as i32).unwrap() - sm)) / self.x_var_sq;
                }
                let alpha = sm - beta * self.x_mean;
                let tsf = alpha + beta * (self.period as f64 + 1.0);
                Some(tsf)
            }
        }
    }

    fn reset(&mut self) {
        self.sma.reset();
        self.history.reset();
    }
}
