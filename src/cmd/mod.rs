extern crate log;

use crate::stdlib::hostname;

use std::path::PathBuf;
use clap::{Args,Parser, Subcommand};
use std::env;
use std::fmt::Debug;
use crate::stdlib;

pub mod setloglevel;
pub mod zabbix_lib;
pub mod zbus_convertkey;
pub mod zbus_gateway;
pub mod zbus_monitor;
pub mod zbus_gateway_processor;
pub mod zbus_gateway_processor_passthrough;
pub mod zbus_gateway_processor_filter;
pub mod zbus_gateway_processor_transformation;
pub mod zbus_gateway_processor_analysis;
pub mod zbus_gateway_processor_prometheus;
pub mod zbus_gateway_stdout_sender;
pub mod zbus_gateway_zbus_sender;
pub mod zbus_gateway_nats_sender;
pub mod zbus_gateway_rhai_sender;
pub mod zbus_gateway_mqtt_sender;
pub mod zbus_gateway_statsd_sender;
pub mod zbus_gateway_telegraf_sender;
pub mod zbus_gateway_clickhouse_sender;
pub mod zbus_gateway_tcpsocket_sender;
pub mod zbus_gateway_catcher_zabbix;
pub mod zbus_gateway_catcher_nats;
pub mod zbus_gateway_catcher_zbus;
pub mod zbus_gateway_catcher_rhai;
pub mod zbus_gateway_catcher_syslogd;
pub mod zbus_gateway_catcher_prometheus_scraper;
pub mod zbus_version;
pub mod zbus_login;
pub mod zbus_json;
pub mod zbus_rhai;
pub mod zbus_sampler;

pub fn init() {
    log::debug!("Parsing CLI parameters");
    let cli = Cli::parse();
    setloglevel::setloglevel(&cli);
    stdlib::initlib(&cli);

    match &cli.command {
        Commands::Gateway(gateway) => {
            log::debug!("Execute ZBUSDG");
            zbus_rhai::run(&cli, &gateway);
            zbus_gateway::run(&cli, &gateway);
        }
        Commands::Monitor(monitor) => {
            log::debug!("Execute ZBUS Monitor");
            zbus_monitor::run(&cli, &monitor);
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
#[clap(name = "zbusudg")]
#[clap(author = "Vladimir Ulogov <vladimir@ulogov.us>")]
#[clap(version = env!("CARGO_PKG_VERSION"))]
#[clap(about = "ZBUS Universal Data Gateway", long_about = None)]
pub struct Cli {
    #[clap(short, long, action = clap::ArgAction::Count, help="Increase verbosity")]
    pub debug: u8,

    #[clap(help="ZBUS telemetry protocol version", long, default_value_t = String::from("v2"))]
    pub protocol_version: String,

    #[clap(help="ID of the observability platform", long, default_value_t = String::from("local"))]
    pub platform_name: String,

    #[clap(help="Telemetry source", long, default_value_t = String::from(hostname::get_hostname()))]
    pub source: String,

    #[clap(help="Telemetry route", long, default_value_t = String::from("local"))]
    pub route: String,


    #[clap(help="Authentication token", long, default_value_t = String::from(""))]
    pub token: String,

    #[clap(help="Zabbix API endpoint", long, default_value_t = String::from("http://127.0.0.1:8080"))]
    pub zabbix_api: String,

    #[clap(long, default_value_t = 16, help="Number of threads in ThreadManager")]
    pub threads: u16,

    #[clap(long, default_value_t = 3600, help="Timeout for Zabbix ITEMS cache")]
    pub item_cache_timeout: u16,

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
#[clap(about="Monitor ZBUS key")]
pub struct Monitor {
    #[clap(help="ZBUS address", long, default_value_t = String::from(env::var("ZBUS_ADDRESS").unwrap_or("tcp/127.0.0.1:7447".to_string())))]
    pub zbus_connect: String,

    #[clap(help="ZBUS listen address", long, default_value_t = String::from_utf8(vec![]).unwrap())]
    pub zbus_listen: String,

    #[clap(help="ZBUS monitor key", long, default_value_t = String::from("zbus/metric/v2/local/aggregation"))]
    pub zbus_key: String,

    #[clap(long, action = clap::ArgAction::SetTrue, help="Disable multicast discovery of ZENOH bus")]
    pub zbus_disable_multicast_scout: bool,

    #[clap(long, action = clap::ArgAction::SetTrue, help="Configure CONNECT mode for ZENOH bus")]
    pub zbus_set_connect_mode: bool,

}

#[derive(Args, Clone, Debug)]
#[clap(about="Execute ZBUS Universal Data Gateway")]
pub struct Gateway {

    #[arg(short, long, value_name = "SCRIPT")]
    script: Option<PathBuf>,

    #[clap(help="Zabbix AUTH token", long, default_value_t = String::from(""))]
    pub zabbix_token: String,

    #[clap(help="Listen address for the stream catcher", long, default_value_t = String::from("0.0.0.0:10055"))]
    pub listen: String,

    #[clap(long, default_value_t = 1, help="Number of catcher threads")]
    pub threads: u16,

    #[clap(long, action = clap::ArgAction::SetTrue, help="Add analythical collection and capabilities to the in-line telemetry processing")]
    pub analysis: bool,

    #[clap(long, action = clap::ArgAction::SetTrue, help="Monitor elapsed time for JSON batch processing")]
    pub telemetry_monitor_elapsed: bool,

    #[clap(long, action = clap::ArgAction::SetTrue, help="Display a pretty JSON")]
    pub pretty: bool,

    #[clap(help="Destination address for raw TCP sender", long, default_value_t = String::from("127.0.0.1:55554"))]
    pub tcp_connect: String,

    #[clap(help="ZBUS address", long, default_value_t = String::from(env::var("ZBUS_ADDRESS").unwrap_or("tcp/127.0.0.1:7447".to_string())))]
    pub zbus_connect: String,

    #[clap(help="ZBUS address for the catcher", long, default_value_t = String::from(env::var("ZBUS_CATCH_ADDRESS").unwrap_or("tcp/127.0.0.1:7447".to_string())))]
    pub zbus_catcher_connect: String,

    #[clap(help="NATS address", long, default_value_t = String::from(env::var("NATS_ADDRESS").unwrap_or("127.0.0.1:4222".to_string())))]
    pub nats_connect: String,

    #[clap(help="MQTT address", long, default_value_t = String::from(env::var("MQTT_ADDRESS").unwrap_or("127.0.0.1:1883".to_string())))]
    pub mqtt_connect: String,

    #[clap(help="ZBUS listen address", long, default_value_t = String::from_utf8(vec![]).unwrap())]
    pub zbus_listen: String,

    #[clap(help="ZBUS aggregate key", long, default_value_t = String::from("aggregation"))]
    pub zbus_aggregate_key: String,

    #[clap(help="ZBUS key from which catcher will receive telemetry", long, default_value_t = String::from("aggregation"))]
    pub zbus_subscribe_key: String,

    #[clap(help="NATS aggregate key", long, default_value_t = String::from("aggregation"))]
    pub nats_aggregate_key: String,

    #[clap(help="NATS subscribe key", long, default_value_t = String::from("aggregation"))]
    pub nats_subscribe_key: String,

    #[clap(help="MQTT aggregate key", long, default_value_t = String::from("aggregation"))]
    pub mqtt_aggregate_key: String,

    #[clap(help="STATSD address", long, default_value_t = String::from("127.0.0.1:8125"))]
    pub statsd_connect: String,

    #[clap(help="TELEGRAF address", long, default_value_t = String::from("tcp://localhost:8094"))]
    pub telegraf_connect: String,

    #[clap(help="CLICKHOUSE address", long, default_value_t = String::from("http://127.0.0.1:8123/?"))]
    pub clickhouse_connect: String,

    #[clap(help="CLICKHOUSE database", long, default_value_t = String::from("zbus"))]
    pub clickhouse_database: String,

    #[clap(help="Prometheus exporter endpoints", long)]
    pub prometheus_exporter_connect: Vec<String>,

    #[clap(long, action = clap::ArgAction::SetTrue, help="Disable multicast discovery of ZENOH bus")]
    pub zbus_disable_multicast_scout: bool,

    #[clap(long, action = clap::ArgAction::SetTrue, help="Aggregate all keys to a single ZBUS topic")]
    pub zbus_aggregate: bool,

    #[clap(long, action = clap::ArgAction::SetTrue, help="Send two copies of the telemetry: one to aggregated topic another to split topic")]
    pub zbus_aggregate_and_split: bool,

    #[clap(long, action = clap::ArgAction::SetTrue, help="Aggregate all keys to a single NATS subject")]
    pub nats_aggregate: bool,

    #[clap(long, action = clap::ArgAction::SetTrue, help="Configure CONNECT mode for ZENOH bus")]
    pub zbus_set_connect_mode: bool,

    #[clap(long, default_value_t = 5, help="TCP timeout for raw TCP sender")]
    pub tcp_timeout: u16,

    #[clap(long, default_value_t = 7, help="Width of anomalies window")]
    pub anomalies_window: usize,

    #[clap(long, default_value_t = 120, help="Delay (in seconds) between prometheus scraper run")]
    pub prometheus_scraper_run_every: u16,

    #[clap(long, default_value_t = 5, help="Delay (in seconds) between running RHAI catcher function")]
    pub rhai_catcher_run_every: u16,

    #[clap(long, default_value_t = 514, help="UDP port for syslogd catcher")]
    pub syslogd_udp_port: u16,

    #[clap(long, default_value_t = 1024, help="SYSLOGD catcher capacity")]
    pub syslogd_catcher_capacity: u16,

    #[clap(help="SYSLOGD key", long, default_value_t = String::from("zbus/log/syslog"))]
    pub syslogd_key: String,

    #[clap(flatten)]
    catchers: CatcherArgGroup,

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

    #[clap(long, action = clap::ArgAction::SetTrue, help="Send catched data to ZBUS")]
    pub zbus: bool,

    #[clap(long, action = clap::ArgAction::SetTrue, help="Send catched data to NATS")]
    pub nats: bool,

    #[clap(long, action = clap::ArgAction::SetTrue, help="Send catched data to MQTT")]
    pub mqtt: bool,

    #[clap(long, action = clap::ArgAction::SetTrue, help="Send catched data to STATSD")]
    pub statsd: bool,

    #[clap(long, action = clap::ArgAction::SetTrue, help="Send catched data to TELEGRAF")]
    pub telegraf: bool,

    #[clap(long, action = clap::ArgAction::SetTrue, help="Send catched data to CLICKHOUSE")]
    pub clickhouse: bool,

    #[clap(long, action = clap::ArgAction::SetTrue, help="Send catched data to a RHAI script")]
    pub rhai: bool,

    #[clap(long, action = clap::ArgAction::SetTrue, help="Send catched data to NONE")]
    pub none: bool,
}

#[derive(Debug, Clone, clap::Args)]
#[group(required = true, multiple = false)]
pub struct CatcherArgGroup {
    #[clap(long, action = clap::ArgAction::SetTrue, help="Catch telemetry from Zabbix")]
    pub zabbix: bool,

    #[clap(long, action = clap::ArgAction::SetTrue, help="Catch telemetry from NATS")]
    pub nats_catcher: bool,

    #[clap(long, action = clap::ArgAction::SetTrue, help="Catch telemetry from ZBUS")]
    pub zbus_catcher: bool,

    #[clap(long, action = clap::ArgAction::SetTrue, help="Receive telemetry from Prometheus scraper")]
    pub prometheus_exporter_catcher: bool,

    #[clap(long, action = clap::ArgAction::SetTrue, help="Generate telemetry data by the script")]
    pub rhai_catcher: bool,

    #[clap(long, action = clap::ArgAction::SetTrue, help="Running syslogd catcher")]
    pub syslogd_catcher: bool,

}

#[derive(Subcommand, Clone, Debug)]
enum Commands {
    Login(Login),
    ConvertKey(ConvertKey),
    Gateway(Gateway),
    Monitor(Monitor),
    Version(Version),
}
