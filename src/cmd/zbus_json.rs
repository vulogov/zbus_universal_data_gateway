extern crate log;
use crate::cmd;
use crate::stdlib;
use nanoid::nanoid;
use serde_json::{json, Value};

pub fn generate_json_telemetry(c: &cmd::Cli, dst: String, name: String, ctype: usize, data: Value) -> Value {
    let ts = stdlib::time::timestamp_ns();
    json!({
        "headers": {
            "messageType":      "telemetry",
            "route":            c.route.clone(),
            "streamName":       c.platform_name.clone(),
            "cultureCode":      null,
            "version":          c.protocol_version.clone(),
            "encryptionAlgorithm":      null,
            "compressionAlgorithm":     null,
        },
        "body": {
            "details": {
                "origin":       c.platform_name.clone(),
                "destination":  dst.clone(),
                "properties":   {
                    "zabbix_clock":     stdlib::time::whole_seconds(ts),
                    "zabbix_ns":        stdlib::time::nanoseconds(ts),
                    "zabbix_host_name": c.source.clone(),
                    "zabbix_itemid":    null,
                    "name":             name.clone(),
                    "tags":             null,

                },
                "details":  {
                    "detailType":   "",
                    "contentType":  ctype,
                    "data":         data,
                }
            }
        },
        "id": nanoid!(),
    })
}
