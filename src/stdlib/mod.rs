extern crate log;

pub mod banner;
pub mod hostname;
pub mod channel;
pub mod sleep;
pub mod threads;
pub mod time;
pub mod zio;
pub mod tsf;
pub mod forecast_oscillator;
pub mod fixedqueue;
pub mod traits;
pub mod syslog;
pub mod zabbix;

use crate::cmd::{Cli};


pub fn initlib(c: &Cli) {
    log::trace!("Running STDLIB init");
    channel::pipes_init();
    threads::threads_init(c);
}
