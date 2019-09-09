use std::{
    io::self,
    thread,
    time::{
        Duration,
        SystemTime,
        SystemTimeError,
    }
};
use snmp::{
    SyncSession,
    Value
};

use tsdb::{
    Sql,
    SqlError,
    SqlOperations,
    Report,
};

use crate::config::Config;


#[derive(Debug, Error)]
pub enum SNMPError {
    #[error_from("Config IO: {}", 0)]
    Io(io::Error),
    #[error_from("Config: {}", 0)]
    Sql(SqlError),
    #[error_from("Config: {}", 0)]
    SystemTime(SystemTimeError),
}


pub type Result<T> = std::result::Result<T, SNMPError>;


const TIMEOUT: Duration = Duration::from_secs(2);
const LOOP_TIME: Duration = Duration::from_secs(8);


pub fn snmp_loop(config: &Config) -> Result<()> {
    loop {
        for device in &config.devices {
            let agent_addr = &format!("{}:161", &device.ip);
            let community = device.community.as_bytes();
            for mib in &device.mibs {
                println!("input - {:?}", &mib.oid);

                let mut sess = match SyncSession::new(agent_addr, community, Some(TIMEOUT), 0) {
                    Ok(v) => v,
                    _ => continue,
                };

                let mut response = match sess.get(&mib.oid) {
                    Ok(v) => v,
                    _ => continue,
                };

                if let Some((_oid, Value::Counter32(snmp_response))) = response.varbinds.next() {
                    println!("output: {}", snmp_response);
                    let unix_time = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH)?;

                    let mut report = Report::default();
                    report.data_start = unix_time.as_secs() as i64;
                    report.data = snmp_response as i64;
                    report.id_parameter = mib.id_db;
                    report.push_sql()?;
                }
            }
        }
        thread::sleep(LOOP_TIME);
    }
    Ok(())
}
