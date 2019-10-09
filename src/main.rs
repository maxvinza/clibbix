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

mod output;
pub use crate::output::*;


fn main() {
    let mut config = Config::new().unwrap();
    config.load_json().unwrap();

    let mut printoption = PrintOption::new().unwrap();

    let mut wait_loop_time = false;
    let mut wait_dev_name = false;
    let mut wait_parameter_name = false;
    let mut wait_print_time = false;

    for arg in env::args() {
        let argument = arg.as_str();
        match argument {
            "-h" => {
                println!("clibbix - lite monitoring system\n\
                Usage for write: clibbix -i 60 [-config]\n\
                where 60 - polling interval in seconds\n\
                Usage for reading: clibbix -dev 192.168.88.1 [-p test] -t 60\n\
                where dev - ip or name device\n\
                -p (optional) - parameter name\n\
                -t time interval in minits or [day, week, month, year]\n");
                process::exit(1);
            },
            "-dev" => wait_dev_name = true,
            "-i" => wait_loop_time = true,
            "-t" => wait_print_time = true,
            "-p" => wait_parameter_name = true,
            _ => {
                let mut parsed = argument.parse::<usize>().unwrap_or(0);
                if parsed > 0 && wait_loop_time {
                    config.loop_time = parsed;
                    println!("INFO: set up loop_time {}", &config.loop_time);
                    wait_loop_time = false;
                } else if wait_dev_name {
                    printoption.need_print = true;
                    printoption.device = arg;
                    wait_dev_name = false;
                } else if wait_parameter_name {
                    printoption.parameter = arg;
                    wait_parameter_name = false;
                } else if wait_print_time {
                    if parsed == 0 {
                        parsed = match argument {
                            "day" => 1440,
                            "week" => 10080,
                            "month" => 43200,
                            "year" => 518400,
                            _ => 60,
                        }
                    }
                    printoption.set_time(parsed).unwrap();
                    wait_print_time = false;
                }
            }
        }
    }

    match printoption.need_print {
        true => printoption.print_data(&mut config),
        false => snmp_loop(&mut config).unwrap(),
    }
}
