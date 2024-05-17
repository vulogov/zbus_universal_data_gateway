extern crate log;
use crate::cmd;
use crate::stdlib::banner;


pub fn run(_c: &cmd::Cli)  {
    log::trace!("zbus_version::run() reached");
    println!("{}", banner::bund_banner());
}
