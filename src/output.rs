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
    SqlError,
    SqlOperations,
    Report,
};

use crate::config::{
    Config,
    Mib,
};


#[derive(Debug, Error)]
pub enum OutputError {
    #[error_from("Output IO: {}", 0)]
    Io(io::Error),
    #[error_from("Output: {}", 0)]
    Sql(SqlError),
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
    first_time: i64,
    lats_time: i64,
    num_reports: i64,
}


impl Default for PrintOption {
    fn default() -> Self {
        Self {
            need_print: false,
            device: String::default(),
            parameter: String::default(),
            first_time: 1,
            lats_time: 1,
            num_reports: 10,
        }
    }
}


impl PrintOption {
    pub fn set_time(&mut self, time: i64) -> Result<()> {
        let unix_time = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH)?;
        self.lats_time = unix_time.as_secs().try_into()?;
        self.first_time = self.lats_time - time * 60;
        Ok(())
    }

    pub fn print_data(&mut self, config: &Config) {
        let ip = self.device.as_str();
        let p = self.parameter.clone();
        let parameter = p.as_str();

        if self.need_print {
            for device in &config.devices {
                if ip == device.ip.as_str() {
                    for mib in &device.mibs {
                        if parameter == "" {
                            println!("Device: {}", device.ip);
                            if let Err(_e) = self.printreport(mib) { break; }
                        } else if parameter == &mib.name {
                            println!("Device: {}", device.ip);
                            if let Err(_e) = self.printreport(mib) { break; }
                        }
                    }
                    break;
                }
            }
        }
    }

    pub fn printreport(&mut self, mib: &Mib) -> Result<()> {
        println!("Parameter: {}", mib.name);
        let mut last_report_time = 0;
        let delta_time = (self.lats_time - self.first_time) / self.num_reports;
        for n in 0 .. self.num_reports - 1 {
            let mut report = Report::default();
            report.id_parameter = mib.id_db;
            report.data_start = self.first_time + delta_time * (n as i64);
            report.pull_sql_up()?;

            if last_report_time != report.data_start {
                let devision = mib.devision;

                let d = UNIX_EPOCH + Duration::from_secs(report.data_start.try_into()?);
                let datetime = DateTime::<Utc>::from(d);
                // Formats the combined date and time with the specified format string.
                let timestamp_str = datetime.format("%Y-%m-%d %H:%M:%S").to_string();
                println!("{} -> {}", timestamp_str, report.data / devision);
                last_report_time = report.data_start;
            }
        }
        Ok(())
    }
}
