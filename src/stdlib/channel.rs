extern crate log;
use easy_error::{bail, Error};
use lazy_static::lazy_static;
use std::sync::Mutex;
use std::collections::btree_map::BTreeMap;
use crossbeam::channel::{Sender, Receiver, unbounded};

lazy_static! {
    static ref PIPES: Mutex<BTreeMap<String,(Sender<String>, Receiver<String>)>> = {
        let e: Mutex<BTreeMap<String,(Sender<String>, Receiver<String>)>> = Mutex::new(BTreeMap::new());
        e
    };
}

pub fn pipes_init() {
    log::debug!("Initializing default pipes");
    let mut q = PIPES.lock().unwrap();
    q.insert("in".to_string(), unbounded::<String>());
    q.insert("out".to_string(), unbounded::<String>());
    q.insert("filter".to_string(), unbounded::<String>());
    q.insert("transformation".to_string(), unbounded::<String>());
    drop(q);
}

pub fn create_pipe(n: String) {
    log::debug!("Create pipe: {}", &n);
    let mut q = PIPES.lock().unwrap();
    q.insert(n.to_string(), unbounded::<String>());
    drop(q);
}

pub fn pipe_is_empty(k: String) -> Result<bool, Box<Error>> {
    let mut q = PIPES.lock().unwrap();
    if ! q.contains_key(&k) {
        drop(q);
        bail!("bus has no pipe: {}", &k);
    }
    let (_, r) = q.get_mut(&k).unwrap();
    if r.is_empty() {
        drop(q);
        return Result::Ok(true);
    }
    drop(q);
    Result::Ok(false)
}

pub fn pipe_is_empty_raw(k: String) -> bool {
    let mut q = PIPES.lock().unwrap();
    if ! q.contains_key(&k) {
        drop(q);
        return false;
    }
    let (_, r) = q.get_mut(&k).unwrap();
    if r.is_empty() {
        drop(q);
        return true;
    }
    drop(q);
    false
}

pub fn pipe_push(k: String, d: String) {
    let mut q = PIPES.lock().unwrap();
    if ! q.contains_key(&k) {
        log::trace!("new bus::internal::pipe : {}", &k);
        let (s,r) = unbounded::<String>();
        match s.send(d) {
            Ok(_) => {
                q.insert(k, (s,r));
            }
            Err(_) => {
                drop(q);
            }
        };
    } else {
        let (s,_) = q.get_mut(&k).unwrap();
        match s.send(d) {
            Ok(_) => {},
            Err(_) => {
                drop(q);
            }
        }
    }
}

pub fn pipe_pull(k: String) -> Result<String, Box<Error>> {
    let mut q = PIPES.lock().unwrap();
    if ! q.contains_key(&k) {
        drop(q);
        bail!("bus::internal::pipe no pipe: {}", &k);
    }
    let (_, r) = q.get_mut(&k).unwrap();
    if r.is_empty() {
        bail!("bus::internal::pipe is empty: {}", &k);
    }
    match r.recv() {
        Ok(res) => {
            return Result::Ok(res);
        }
        Err(err) => bail!("bus::internal::pipe {} can not recv: {}", &k, &err),
    }
}
