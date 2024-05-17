extern crate log;
use lazy_static::lazy_static;
use std::sync::Mutex;
use thread_manager::ThreadManager;
use crate::cmd;


lazy_static! {
    pub static ref THREADS: Mutex<ThreadManager<()>> = {
        let e: Mutex<ThreadManager<()>> = Mutex::new(ThreadManager::<()>::new(4));
        e
    };
}

pub fn terminale_all() {
    let t = THREADS.lock().unwrap();
    log::debug!("Terimating managed threads");
    t.terminate_all();
    drop(t);
}

pub fn threads_init(c: &cmd::Cli) {
    log::trace!("Running STDLIB::threads init");
    let mut t = THREADS.lock().unwrap();
    log::debug!("Thread engine has been configured with {} threads", &c.threads);
    t.resize(c.threads as usize);
    drop(t);
}
