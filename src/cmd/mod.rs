extern crate log;

use crate::stdlib::hostname;

use clap::{Args,Parser, Subcommand};
use std::str::FromStr;
use std::env;
use std::fmt::Debug;
use crate::stdlib;

pub mod setloglevel;
pub mod zabbix_lib;
pub mod zbus_convertkey;
pub mod zbus_gateway;
pub mod zbus_gateway_processor;
pub mod zbus_gateway_stdout_sender;
pub mod zbus_version;
pub mod zbus_login;

pub fn init() {
    log::debug!("Parsing CLI parameters");
    let cli = Cli::parse();
    setloglevel::setloglevel(&cli);
    stdlib::initlib(&cli);

    match &cli.command {
        Commands::Gateway(gateway) => {
            log::debug!("Execute ZBUSDG");
            zbus_gateway::run(&cli, &gateway);
        }
        Commands::ConvertKey(convertkey) => {
            log::debug!("Generate ZabbixAPI token");
            zbus_convertkey::run(&cli, &convertkey);
        }
        Commands::Login(login) => {
            log::debug!("Generate ZabbixAPI token");
            zbus_login::run(&cli, &login);
        }
        Commands::Version(_) => {
            log::debug!("Get the tool version");
            zbus_version::run(&cli);
        }
    }
}

#[derive(Parser, Clone)]
#[clap(name = "zbus")]
#[clap(author = "Vladimir Ulogov <vladimir@ulogov.us>")]
#[clap(version = env!("CARGO_PKG_VERSION"))]
#[clap(about = "ZBUS telemetry CLI tool", long_about = None)]
pub struct Cli {
    #[clap(short, long, action = clap::ArgAction::Count, help="Increase verbosity")]
    pub debug: u8,

    #[clap(help="ZBUS telemetry protocol version", long, default_value_t = String::from("v1"))]
    pub protocol_version: String,

    #[clap(help="ID of the observability platform", long, default_value_t = String::from("local"))]
    pub platform_name: String,

    #[clap(help="Telemetry source", long, default_value_t = String::from(hostname::get_hostname()))]
    pub source: String,

    #[clap(help="Authentication token", long, default_value_t = String::from(""))]
    pub token: String,

    #[clap(help="Zabbix API endpoint", long, default_value_t = String::from("http://127.0.0.1:8080"))]
    pub zabbix_api: String,

    #[clap(help="Listen address for the stream catcher", long, default_value_t = String::from("0.0.0.0:10055"))]
    pub listen: String,

    #[clap(long, default_value_t = 16, help="Number of threads in ThreadManager")]
    pub threads: u16,

    #[clap(subcommand)]
    command: Commands,
}

#[derive(Args, Clone, Debug)]
#[clap(about="Get the version of the tool")]
pub struct Version {
    #[clap(last = true)]
    args: Vec<String>,
}

#[derive(Args, Clone, Debug)]
#[clap(about="Generate Zabbix API token")]
pub struct Login {
    #[clap(help="Zabbix API username", long, default_value_t = String::from("Admin"))]
    pub zabbix_username: String,

    #[clap(help="Zabbix API password", long, default_value_t = String::from("zabbix"))]
    pub zabbix_password: String,
}

#[derive(Args, Clone, Debug)]
#[clap(about="Convert Zabbix key")]
pub struct ConvertKey {
    #[clap(help="Zabbix key", long, default_value_t = String::from("agent.ping"))]
    pub key: String,
}

#[derive(Args, Clone, Debug)]
#[clap(about="Execute ZBUS Universal Data Gateway")]
pub struct Gateway {
    #[clap(help="Listen address for the stream catcher", long, default_value_t = String::from("0.0.0.0:10055"))]
    pub listen: String,

    #[clap(long, default_value_t = 1, help="Number of catcher threads")]
    pub threads: u16,

    #[clap(flatten)]
    group: GatewayArgGroup,
}

#[derive(Debug, Clone, clap::Args)]
#[group(required = true, multiple = false)]
pub struct GatewayArgGroup {
    #[clap(long, action = clap::ArgAction::SetTrue, help="Send catched data to STDOUT")]
    pub stdout: bool,

    #[clap(long, action = clap::ArgAction::SetTrue, help="Send catched data to RAW socket")]
    pub socket: bool,
}

#[derive(Subcommand, Clone, Debug)]
enum Commands {
    Login(Login),
    ConvertKey(ConvertKey),
    Gateway(Gateway),
    Version(Version),
}
