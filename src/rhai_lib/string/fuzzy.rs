extern crate log;
use fuzzy_matcher::FuzzyMatcher;
use fuzzy_matcher::skim::SkimMatcherV2;
use distance::*;
use rhai::{Dynamic, NativeCallContext, EvalAltResult};

pub fn str_match_raw(t: String, p: String) -> i64 {
    let matcher = SkimMatcherV2::default();
    match matcher.fuzzy_match(&t, &p) {
        Some(res) => res as i64,
        None => 0 as i64,
    }
}

pub fn str_match_levenshtein_raw(t: String, p: String) -> i64 {
    levenshtein(&t, &p) as i64
}

pub fn str_match(_context: NativeCallContext, t: String, p: String) -> Result<Dynamic, Box<EvalAltResult>> {
    Result::Ok(Dynamic::from(str_match_raw(t, p)))
}

pub fn str_match_levenshtein(_context: NativeCallContext, t: String, p: String) -> Result<Dynamic, Box<EvalAltResult>> {
    Result::Ok(Dynamic::from(str_match_levenshtein_raw(t.clone(), p.clone()) as i64))
}

pub fn str_match_damerau(_context: NativeCallContext, t: String, p: String) -> Result<Dynamic, Box<EvalAltResult>> {
    Result::Ok(Dynamic::from(damerau_levenshtein(&t, &p) as i64))
}
