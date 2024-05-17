extern crate log;

pub mod banner;
pub mod hostname;
pub mod channel;
pub mod sleep;
pub mod threads;

use crate::cmd::{Cli};


pub fn initlib(c: &Cli) {
    log::trace!("Running STDLIB init");
    channel::pipes_init();
    threads::threads_init(c);
}
