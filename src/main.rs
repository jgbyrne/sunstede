mod config;
mod generate;
use std::env;
use std::process;

use config::{Config, ConfigError};
use generate::generate;

fn help() {
    eprintln!("sunstede -C /path/to/config.jacl");
}

fn die(e: ConfigError) -> ! {
    match e {
        ConfigError::Filesystem(msg) => eprintln!("{}", msg),
        ConfigError::Jacl(msg) => eprintln!("{}", msg),
        ConfigError::Logical(msg) => eprintln!("{}", msg),
    }
    process::exit(2);
}

fn main() {
    let mut args = env::args();
    let config = match args.len() {
        0 => {
            eprintln!("Recieved no arguments");
            process::exit(1);
        },
        1 => { 
            match Config::from_file("config.jacl") {
                Ok(conf) => conf,
                Err(e) => die(e),
            }
        },
        2 => {
            match Config::from_file(&args.nth(1).unwrap()) {
                Ok(conf) => conf,
                Err(e) => die(e),
            }
        },
        _ => {
            help();
            process::exit(1);
        }
    };
    
    for site in config.sites {
        generate(site);
    }
}
