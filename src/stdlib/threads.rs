extern crate log;
use lazy_static::lazy_static;
use std::sync::Mutex;
use thread_manager::ThreadManager;
use crate::cmd;
use crate::stdlib;


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

pub fn wait_all() {
    loop {
        let t = THREADS.lock().unwrap();
        let active_threads = t.active_threads();
        if active_threads == 0 {
            drop(t);
            log::debug!("ThreadManager do not have any active threads");
            break;
        }
        log::debug!("{} active threads, {} busy threads and {} waiting threads  in ThreadManager", active_threads, t.busy_threads(), t.waiting_threads());
        drop(t);
        stdlib::sleep::sleep(60);
    }
}

pub fn threads_init(c: &cmd::Cli) {
    log::debug!("Running STDLIB::threads init");
    let mut t = THREADS.lock().unwrap();
    log::debug!("Thread engine has been configured with {} threads", &c.threads);
    t.resize(c.threads as usize);
    drop(t);
}
