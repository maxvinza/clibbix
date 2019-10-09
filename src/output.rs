extern crate chrono;
use chrono::{
    Utc,
    prelude::DateTime,
};
use std::{
    io,
    convert::TryInto,
    num::TryFromIntError,
    time::{
        Duration,
        SystemTime,
        SystemTimeError,
        UNIX_EPOCH,
    }
};
use tsdb::{
    RRDB,
    RRDBError,
};

use crate::config::{
    Device,
    Config,
    Mib,
};


#[derive(Debug, Error)]
pub enum OutputError {
    #[error_from("Output IO: {}", 0)]
    Io(io::Error),
    #[error_from("Output: {}", 0)]
    RRDB(RRDBError),
    #[error_from("Config: {}", 0)]
    SystemTime(SystemTimeError),
    #[error_from("Config: {}", 0)]
    TryFromInt(TryFromIntError),
}


pub type Result<T> = std::result::Result<T, OutputError>;


pub struct PrintOption {
    pub need_print: bool,
    pub device: String,
    pub parameter: String,
    first_time: usize,
    lats_time: usize,
    num_reports: usize,
}


impl PrintOption {
    pub fn new() -> Result<Self> {
        let lats_time = unix_time()? as usize;
        let first_time = lats_time - 3600;
        Ok(Self {
            need_print: false,
            device: String::default(),
            parameter: String::default(),
            first_time,
            lats_time,
            num_reports: 10,
        })
    }

    pub fn set_time(&mut self, time: usize) -> Result<()> {
        self.lats_time = unix_time()? as usize;
        self.first_time = self.lats_time - time * 60;
        Ok(())
    }

    pub fn print_data(&mut self, config: &Config, mut rrdb: RRDB) {
        let ip = self.device.as_str();
        let p = self.parameter.clone();
        let parameter = p.as_str();

        if self.need_print {
            for device in &config.devices {
                if ip == device.ip.as_str() {
                    for mib in &device.mibs {
                        if parameter == "" {
                            println!("Device: {}", device.ip);
                            if let Err(_e) = self.printreport(mib, device, &mut rrdb) { break; }
                        } else if parameter == &mib.name {
                            println!("Device: {}", device.ip);
                            if let Err(_e) = self.printreport(mib, device, &mut rrdb) { break; }
                        }
                    }
                    break;
                }
            }
        }
    }

    pub fn printreport(&mut self, mib: &Mib, device: &Device, rrdb: &mut RRDB) -> Result<()> {
        println!("Parameter: {}", mib.name);
        let mut last_report_time = 0;
        let delta_time = (self.lats_time - self.first_time) / self.num_reports;
        for n in 0 .. self.num_reports - 1 {
            let ts =  (self.first_time + delta_time * n) as u64;
            let report = rrdb.pull_report(mib.id, device.id, ts)?;

            if last_report_time != report.data.start {
                let devision = mib.devision;

                let d = UNIX_EPOCH + Duration::from_secs(report.data.start);
                let datetime = DateTime::<Utc>::from(d);
                // Formats the combined date and time with the specified format string.
                let timestamp_str = datetime.format("%Y-%m-%d %H:%M:%S").to_string();
                println!("{} -> {}", timestamp_str, report.data.data as f32 / devision as f32);
                last_report_time = report.data.start;
            }
        }
        Ok(())
    }
}


fn unix_time() -> Result<i64> {
    let unix_time = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH)?;
    let time: i64 = unix_time.as_secs().try_into()?;
    Ok(time)
}
