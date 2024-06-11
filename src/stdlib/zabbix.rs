extern crate log;
use std::io::{Read, Write};
use serde_json;
use easy_error::{Error, bail};

pub fn zabbix_sender(addr: String, data: serde_json::Value) -> Result<serde_json::Value, Error> {
    match addr.parse::<std::net::SocketAddr>() {
        Ok(address) => {
            match send_metric(address, serde_json::ser::to_vec(&data).unwrap()) {
                Ok(res) => Ok(res),
                Err(err) => bail!("ZabbixSender error: {:?}", err)
            }
        }
        Err(err) => {
            let msg = format!("Error parsing address for ZabbixSender::send(): {}", err);
            log::error!("{}", &msg);
            bail!("{}", &msg);
        }
    }
}

fn send_metric(
    addr: std::net::SocketAddr,
    data: Vec<u8>,
) -> Result<serde_json::Value, Error> {
    let mut buffer = vec![];
    log::trace!("zabbix::zabbix_sender[{}:{}", &addr.ip(), &addr.port());
    let mut sock = match std::net::TcpStream::connect(addr) {
        Ok(sock) => sock,
        Err(err) => bail!("Zabbix::zabbix_sender connect error: {}", err),
    };
    log::trace!("zabbix::zabbix_sender connected");
    match sock.write(
        &[
            "ZBXD\x01".as_bytes(),
            (data.len() as u32).to_le_bytes().as_ref(),
            0u32.to_le_bytes().as_ref(),
            &data,
        ]
        .concat(),
    ) {
        Ok(_) => {},
        Err(err) => bail!("Zabbix::zabbix_sender write error: {}", err),
    }
    match sock.read_to_end(&mut buffer) {
        Ok(_) => {},
        Err(err) => bail!("Zabbix::zabbix_sender read error: {}", err),
    }
    log::trace!("zabbix::zabbix_sender received");
    let _ = sock.shutdown(std::net::Shutdown::Both);
    log::trace!("zabbix::zabbix_sender closed");
    assert_eq!(&buffer[0..5], &[90, 66, 88, 68, 1]);
    let len = u32::from_le_bytes([buffer[5], buffer[6], buffer[7], buffer[8]]) as usize;
    let response = String::from_utf8_lossy(&buffer[13..13 + len]).to_string();
    if response.starts_with("ZBX_NOTSUPPORTED\0") {
        bail!("{}", response.split('\0').nth(1).unwrap_or_default().to_owned())
    } else {
        match serde_json::from_str(&response) {
            Ok(res) => return Ok(res),
            Err(err) => bail!("zabbix::zabbix_sender error converting result: {:?}", err),
        }
    }
}
