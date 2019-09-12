use std::{
    io::{
        self,
        BufRead,
        BufReader,
    },
    fs::File,
};

use tsdb::{
    Sql,
    SqlError,
    SqlOperations,
    Parameter,
    ParameterType,
    Object,
};


#[derive(Debug, Default)]
pub struct Config {
    pub devices: Vec<Device>,
    pub loop_time: i64,
}


#[derive(Debug, Default)]
pub struct Device {
    pub ip: String,
    pub community: String,
    pub mibs: Vec<Mib>,
}

///Stuct of one parameter of monitoring
///devision - coefficient by which reports are divided
#[derive(Debug)]
pub struct Mib {
    pub name: String,
    units: String,
    pub oid: Vec<u32>,
    pub devision: i64,
    pub id_db: i64,
}


impl Default for Mib {
    fn default() -> Self {
        Self {
            name: String::default(),
            units: String::default(),
            oid: Vec::new(),
            devision: 1,
            id_db: 0,
        }
    }
}


#[derive(Debug, Error)]
pub enum ConfigError {
    #[error_from("Config IO: {}", 0)]
    Io(io::Error),
    #[error_from("Config: {}", 0)]
    Sql(SqlError),
}


pub type Result<T> = std::result::Result<T, ConfigError>;


impl Config {
    pub fn new() -> Result<Self> {
        let f = File::open("config.cfg")?;
        let mut reader = BufReader::new(f);
        let mut buffer = String::new();
        let mut device = Device::default();
        let mut is_first = true;
        let mut config = Config::default();
        config.loop_time = 60;//default polling interval
        loop {
            buffer.clear();
            if reader.read_line(&mut buffer)? == 0 {
                break;
            }

            buffer = String::from(buffer.trim());
            if buffer.starts_with("dev") {
                if ! is_first {
                    config.devices.push(device);
                    device = Device::default();
                } else {
                    is_first = false;
                }

                let mut i = buffer[4 ..].split(' ');

                device.ip = String::from(i.next().unwrap_or(""));
                device.community = String::from(i.next().unwrap_or(""));
            } else {
                let mut mib = Mib::default();
                for a in buffer.split(' ') {
                    let mut i = a.split('=');
                    let key = i.next().unwrap_or("");
                    let value = i.next().unwrap_or("");

                    match key {
                        "name" => mib.name = String::from(value),
                        "oid" => {
                            for a in value.split('.') {
                                let elemet = a.parse::<u32>().unwrap_or(0);
                                mib.oid.push(elemet);
                            }
                        },
                        "units" => mib.units = String::from(value),
                        "devision" => mib.devision = value.parse::<i64>().unwrap_or(1),
                        _ => {},
                    }
                }
                device.mibs.push(mib);
            }
        }
        config.devices.push(device);
        Ok(config)
    }

    pub fn make_tsbd(&mut self) -> Result<()> {
        for device in &mut self.devices {
            let mut sql = Sql::new()?;
            sql.new_base();
            let mut object = Object::default();
            object.name = device.ip.clone();
            object.push_sql()?;
            for mib in &mut device.mibs {
                let mut parametertype = ParameterType::default();
                parametertype.name = mib.name.clone();
                parametertype.units = mib.units.clone();
                parametertype.aproxy_time = 60;
                parametertype.push_sql()?;
                let mut parameter = Parameter::new(&parametertype, &object);
                parameter.push_sql()?;
                mib.id_db = parameter.id;
            }
        }
        Ok(())
    }
}
