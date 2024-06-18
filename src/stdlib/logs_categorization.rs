extern crate log;
use lazy_static::lazy_static;
use std::sync::Mutex;
use natural::classifier::NaiveBayesClassifier;

lazy_static! {
    static ref NBC: Mutex<NaiveBayesClassifier> = {
        let m: Mutex<NaiveBayesClassifier> = Mutex::new(NaiveBayesClassifier::new());
        m
    };
}

pub fn nbc_train(label: String, data: String) {
    let mut nbc = NBC.lock().unwrap();
    nbc.train(&data, &label);
    drop(nbc);
}

pub fn nbc_categorize(data: String) -> String {
    let nbc = NBC.lock().unwrap();
    let ret = nbc.guess(&data);
    drop(nbc);
    ret
}
