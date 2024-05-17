extern crate log;
use reqwest;
use serde_json;
use crate::cmd;

fn zabbix_api_login(c: &cmd::Cli, login: &cmd::Login) -> Option<String> {
    match reqwest::blocking::Client::new()
                .post(format!("{}/api_jsonrpc.php", c.zabbix_api))
                .json(&serde_json::json!({
                    "jsonrpc": "2.0",
                    "method": "user.login",
                    "id": 1,
                    "params": {
                        "username": &login.zabbix_username,
                        "password": &login.zabbix_password,
                    }
                }))
                .send() {
        Ok(res) => {
            let jres: serde_json::Value = match res.json() {
                Ok(jres) => jres,
                Err(err) => {
                    log::error!("Error in processing result while user.login: {:?}", err);
                    return None;
                }
            };
            match &jres.get("result") {
                Some(result) => {
                    return Some(result.as_str()?.to_string());
                }
                None => {

                }
            }
        }
        Err(err) => {
            log::error!("Error in sending user.login request: {:?}", err);
        }
    }
    None
}

pub fn run(c: &cmd::Cli, login: &cmd::Login)  {
    log::trace!("zbus_login::run() reached");
    match zabbix_api_login(c, login) {
        Some(token) => {
            println!("{}", token);
        }
        None => {
        }
    }
}
