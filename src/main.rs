#[macro_use]
extern crate error_rules;

mod config;
pub use crate::config::*;

mod snmp;
pub use crate::snmp::*;


fn main() {
    let mut config = Config::new().unwrap();
    config.make_tsbd().unwrap();
    println!("{:#?}", config);
    snmp_loop(&config);
}
