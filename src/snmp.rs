use std::{
    io::self,
    thread,
    convert::TryInto,
    time::{
        Duration,
        SystemTime,
        SystemTimeError,
    }
};
use snmp::{
    SyncSession,
    Value::{
        Integer,
        Counter32,
        Unsigned32,
        Timeticks,
        Counter64,
    },
};

use tsdb::{
    RRDBError,
    Report,
    ReportData,
    ReportId,
};

use crate::config::Config;


#[derive(Debug, Error)]
pub enum SNMPError {
    #[error_from("Config IO: {}", 0)]
    Io(io::Error),
    #[error_from("Config: {}", 0)]
    RRDB(RRDBError),
    #[error_from("Config: {}", 0)]
    SystemTime(SystemTimeError),
}


pub type Result<T> = std::result::Result<T, SNMPError>;


const TIMEOUT: Duration = Duration::from_secs(2);


pub fn snmp_loop(config: &mut Config) -> Result<()> {
    let sleep_time = Duration::from_secs(config.loop_time.try_into().unwrap_or(60));

    loop {
        for device in &config.devices {
            let agent_addr = &format!("{}:161", &device.ip);
            let community = device.community.as_bytes();
            for mib in &device.mibs {
                println!("Device:{}, oid:{}",  device.ip, mib.name);
                
                let mut sess = match SyncSession::new(agent_addr, community, Some(TIMEOUT), 0) {
                    Ok(v) => v,
                    _ => continue,
                };

                let mut response = match sess.get(&mib.oid) {
                    Ok(v) => v,
                    _ => continue,
                };

                if let Some((_oid, value)) = response.varbinds.next() {
                    //Value::Counter32(snmp_response)
                    let snmp_response: i64 = match value {
                        Integer(v) => v,
                        Counter32(v) => v as i64,
                        Unsigned32(v) => v as i64,
                        Timeticks(v) => v as i64,
                        Counter64(v) => v as i64,
                        _ => 0,
                    };

                    let unix_time = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH)?;

                    let report = Report {
                        id: ReportId {
                            parameter: mib.id,
                            object: device.id,
                        },
                        data: ReportData {
                            start: unix_time.as_secs(),
                            data: snmp_response,
                        }
                    };

                    config.rrdb.push_report(report)?;
                }
            }
        }

        thread::sleep(sleep_time);
    }
}
