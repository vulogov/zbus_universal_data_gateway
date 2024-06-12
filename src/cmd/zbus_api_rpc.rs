extern crate log;
use crate::cmd;
use crate::stdlib;
use std::net::SocketAddr;

use jsonrpc_http_server::jsonrpc_core::{IoHandler};
use jsonrpc_http_server::ServerBuilder;
use jsonrpc_core::Result;
use jsonrpc_derive::rpc;

#[rpc]
pub trait Rpc {
	#[rpc(name = "version")]
	fn version(&self) -> Result<String>;
    #[rpc(name = "last")]
	fn last(&self, key: String) -> Result<serde_json::Value>;
    #[rpc(name = "sample")]
	fn sample(&self, key: String) -> Result<serde_json::Value>;
    #[rpc(name = "metrics")]
	fn metrics(&self) -> Result<serde_json::Value>;
}

pub struct RpcImpl;
impl Rpc for RpcImpl {
	fn version(&self) -> Result<String> {
		Ok(env!("CARGO_PKG_VERSION").to_string())
	}
    fn last(&self, key: String) -> Result<serde_json::Value> {
		match cmd::zbus_api::get_metric(key.clone()) {
            Some(samples) => {
                match samples.last() {
                    Some(val) => return Ok(val),
                    None => Ok(serde_json::json!(null)),
                }
            }
            None => {
                return Ok(serde_json::json!(null));
            }
        }
	}
    fn sample(&self, key: String) -> Result<serde_json::Value> {
		match cmd::zbus_api::get_metric(key.clone()) {
            Some(samples) => {
                return Ok(serde_json::json!(samples.data()));
            }
            None => {
                return Ok(serde_json::json!(null));
            }
        }
	}
    fn metrics(&self) -> Result<serde_json::Value> {
		return Ok(serde_json::json!(cmd::zbus_api::get_keys()))
	}
}

pub fn run(_c: &cmd::Cli, apicli: &cmd::Api)  {
    log::debug!("zbus_api_rpc::run() reached");

    let apicli = apicli.clone();

    match stdlib::threads::THREADS.lock() {
        Ok(t) => {
            t.execute(move ||
            {
                let mut io = IoHandler::default();
                io.extend_with(RpcImpl.to_delegate());
                let addr: SocketAddr = match apicli.api_listen.parse() {
                    Ok(addr) => addr,
                    Err(err) => {
                        log::error!("Error parsing listen address: {}", err);
                        return;
                    }
                };
                let server = match ServerBuilder::new(io)
                                    .threads(apicli.server_threads.into())
                                    .start_http(&addr) {
                    Ok(server) => server,
                    Err(err) => {
                        log::error!("Error starting JSON-RPC server: {}", err);
                        return;
                    }
                };
                log::debug!("Server waiting");
                server.wait();
            });
        }
        Err(err) => {
            log::error!("Error accessing Thread Manager: {:?}", err);
            return;
        }
    }
}
