extern crate log;
use crate::cmd;



pub fn run(_c: &cmd::Cli, convertkey: &cmd::ConvertKey)  {
    log::trace!("zbus_convertkey::run() reached");
    match cmd::zabbix_lib::zabbix_key_to_zenoh(convertkey.key.clone()) {
        Some(key) => {
            println!("{}", key);
        }
        None => {
        }
    }
}
