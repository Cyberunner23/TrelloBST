
use std::fs::{File, Metadata};
use std::io::Read;
use std::path::{Path, PathBuf};
use std::process::exit;

extern crate clap;
extern crate rustc_serialize;

use clap::{Arg, App};
use rustc_serialize::json;

mod appveyor;
mod config;
mod travis_ci;
mod trello;


fn parse_config_file(file: &mut File) -> Result<config::TrelloBSTAPIConfig, &'static str>{
    //TODO: make array, read, parse, return config struct.
    let metadata: Metadata;
    match file.metadata() {
        Ok(_metadata)  => {
            metadata = _metadata;
        }
        Err(err) => {
            return Err("Cannot gather metadata of the configuration file, configuration file won't be used...")
        }
    }

    let file_length:   usize = metadata.len() as usize;
    let api_config   = config::TrelloBSTAPIConfig {
        trello_api_key:      "".to_string(),
        trello_app_key:      "".to_string(),
        travis_access_token: "".to_string(),
        appveyor_api_token:  "".to_string()
    };

    if file_length == 0 {
        return Ok(api_config)
    } else {
        let mut data: String = String::with_capacity(file_length +1);
        match file.read_to_string(&mut data) {
            Ok(size) => {
                //TODO: better error checking
                return Ok(json::decode(&data[..]).unwrap())
            },
            Err(err) => {
                return Err("Error while reading the configuration file, configuration file won't be used...")
            }
        }
    }
}


fn main() {

    let trellobst_version = "0.0.1";

    println!("|----------------------------------------------------------|");
    println!("|-------- Welcome to the Trello Build Status Tool. --------|");
    println!("|----------------------------------------------------------|");


    ////////////////////////////////////////////////////////////
    //               Parse command line options               //
    ////////////////////////////////////////////////////////////

    let mut is_using_config_file   = true;
    let mut is_using_custom_config = false;

    let matches = App::new("TrelloBST")
        .version(trellobst_version)
        .arg(Arg::with_name("CONFIG")
            .short("c")
            .long("config")
            .help("Sets a custom config file.")
            .takes_value(true))
        .arg(Arg::with_name("NO-CONFIG")
            .short("n")
            .long("no-config")
            .help("Won't use a config file.")
            .takes_value(false))
        .get_matches();

    if matches.is_present("CONFIG") && matches.is_present("NO-CONFIG") {
        println!("Error: --config (-c) and --no-config (-n) cannot be used at the same time.");
        exit(-1);
    }

    if matches.is_present("NO-CONFIG") {
        is_using_config_file   = false;
    }

    if matches.is_present("CONFIG") {
        is_using_config_file   = true;
        is_using_custom_config = true;
    }


    ////////////////////////////////////////////////////////////
    //                   Get config path                      //
    ////////////////////////////////////////////////////////////

    let mut config_path = config::TrelloBSTConfigPath {
        config_path: PathBuf::new()
    };

    if is_using_config_file && is_using_custom_config {
        println!("Looking for the configuration file: {}", matches.value_of("CONFIG").unwrap());
        let path = Path::new(matches.value_of("CONFIG").unwrap());
        match config::TrelloBSTConfigPath::try_custom_config_path(path) {
            Ok(_config) => {
                config_path = _config;
                println!("Found.");
                is_using_config_file = true;
            },
            Err(c) => {
                println!("An error occured: {}", c);
                println!("Configuration file won't be used...");
                is_using_config_file = false;
            },
        };
    } else if is_using_config_file && !is_using_custom_config {
        println!("Looking for the configuration file in default location...");
        match config::TrelloBSTConfigPath::try_default_config_path() {
            Ok(_config) => {
                config_path = _config;
                println!("Found.");
                is_using_config_file = true;
            },
            Err(c) => {
                println!("An error occured: {}", c);
                println!("Configuration file won't be used...");
                is_using_config_file = false;
            },
        };
    }


    ////////////////////////////////////////////////////////////
    //                     Parse config                       //
    ////////////////////////////////////////////////////////////

    let     config: config::TrelloBSTAPIConfig;
    let mut config_file: File;

    if is_using_config_file {

        println!("Parsing...");

        match File::open(config_path.config_path.as_path()) {
            Ok(file) => {
                config_file = file;
                match parse_config_file(&mut config_file) {
                    Ok(_config) => {
                        config = _config;
                    }
                    Err(err)   => {
                        println!("{}", err);
                        is_using_config_file = false;
                    }
                }
            }
            Err(_)   =>{
                println!("Cannot open config file for parsing, configuration file won't be used...");
                is_using_config_file = false;
            }
        }

        println!("Done.");

    }


}