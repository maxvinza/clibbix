use std::{
    env,
    process,
};

#[macro_use]
extern crate error_rules;

mod config;
pub use crate::config::*;

mod snmp;
pub use crate::snmp::*;


fn main() {
    let mut config = Config::new().unwrap();
    config.make_tsbd().unwrap();
    let mut wait_loop_time = false;
    for argument in env::args() {
        match argument.as_str() {
            "-h" => {
                println!("clibbix - lite monitoring system\n \
                Usage: clibbix -i 60 [-config]\n\
                where 60 - polling interval in seconds\n\
                -config - option to print config file");
                process::exit(1);
            },
            "-config" => println!("{:#?}", &config),
            "-i" => wait_loop_time = true,
            _ => {
                let parsed = argument.parse::<usize>().unwrap_or(0);
                if parsed > 0 && wait_loop_time {
                    config.loop_time = parsed;
                }
                println!("INFO: set up loop_time {}", &config.loop_time);
            }
        }
    }
    snmp_loop(&config).unwrap();
}
