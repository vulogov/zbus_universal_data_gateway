extern crate log;
use crate::cmd;
use crate::stdlib;
use etime::Etime;
extern crate serde;
extern crate serde_hjson;
use serde_hjson::{Map, Value};

pub fn run(c: &cmd::Cli)  {
    log::debug!("zbus_loader_logs_categorization::run() reached");
    match &c.logs_categorization {
        Some(fname) => {
            let data = match stdlib::zio::read_file(fname.clone()) {
                Some(data) => data,
                None => {
                    log::error!("Can not get the logs categorization data");
                    return;
                }
            };
            let mut e = Etime::new();
            e.tic();
            let sample: Map<String, Value> = match serde_hjson::from_str(&data) {
                Ok(sample) => sample,
                Err(err) => {
                    log::error!("Error parsing Logs Categorization dataset: {}", err);
                    return;
                }
            };
            for (k,v) in sample.iter() {
                if v.is_array() {
                    let val = v.as_array().unwrap();
                    log::debug!("Training label: {} with len()={} samples", &k, val.len());
                    for d in val.iter() {
                        stdlib::logs_categorization::nbc_train(k.clone(), d.as_str().unwrap().to_string());
                    }
                }
            }
            let elapsed = e.toc().as_secs_f32();
            log::debug!("Elapsed time for loading and training Naive Bayes Classificator: {} seconds", elapsed);
        }
        None => {
            log::debug!("Logs category training data not provided");
        }
    }

}
