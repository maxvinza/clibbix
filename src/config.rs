use std::{
    io::{
        self,
        BufReader,
        BufWriter
    },
    fs::{
        File,
        OpenOptions,
    }
};
use serde::{
    Deserialize,
    Serialize
};


#[derive(Debug, Default)]
pub struct Config {
    pub devices: Vec<Device>,
    pub loop_time: usize,
}


#[derive(Debug, Default, Serialize, Deserialize)]
pub struct Device {
    pub id: u32,
    pub ip: String,
    pub community: String,
    pub mibs: Vec<Mib>,
}

///Stuct of one parameter of monitoring
///devision - coefficient by which reports are divided
#[derive(Debug, Serialize, Deserialize)]
pub struct Mib {
    pub id: u32,
    pub name: String,
    pub units: String,
    pub oid: Vec<u32>,
    pub devision: i64,
}


impl Default for Mib {
    fn default() -> Self {
        Self {
            id: 0,
            name: String::default(),
            units: String::default(),
            oid: Vec::new(),
            devision: 1,
        }
    }
}


#[derive(Debug, Error)]
pub enum ConfigError {
    #[error_from("Config IO: {}", 0)]
    Io(io::Error),
    #[error_from("Config JSON: {}", 0)]
    JSON(serde_json::error::Error),
}


pub type Result<T> = std::result::Result<T, ConfigError>;


const FILE: &str = "config.json";


impl Config {
    pub fn load_json(&mut self) -> Result<()> {
        let file = match File::open(FILE) {
            Ok(v) => v,
            _ => {
                self.example_config()?;
                return Ok(());
            }
        };

        let reader = BufReader::new(file);
        self.devices = serde_json::from_reader(reader)?;
        self.make_id();
        Ok(())
    }

    fn example_config(&mut self) -> Result<()> {
        let mib = Mib {
            id: 0,
            name: String::from("Uptime"),
            units: String::from("seconds"),
            oid: vec![1, 3, 6, 1, 2, 1, 1, 3],
            devision: 1,
        };

        let device = Device {
            id: 0,
            ip: String::from("127.0.0.1"),
            community: String::from("public"),
            mibs: vec![mib],
        };

        self.devices.push(device);
        self.make_id();
        self.save_json()
    }

    pub fn make_id(&mut self) {
        let mut max_device = 0;
        let mut max_mib = 0;
        for device in &mut self.devices {
            if device.id > max_device { max_device = device.id; }

            for mib in &mut device.mibs {
                if mib.id > max_mib { max_mib = device.id; }
            }
        }

        for device in &mut self.devices {
            if device.id == 0 {
                max_device += 1;
                device.id = max_device;
            }

            for mib in &mut device.mibs {
                if mib.id == 0 { 
                    max_mib += 1;
                    mib.id = max_mib;
                }
            }
        }
    }
    
    pub fn save_json(&self) -> Result<()> {
        let file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open(FILE)?;
        
        let mut writer = BufWriter::new(file);
        serde_json::to_writer(&mut writer, &self.devices)?;
        Ok(())
    }
}
