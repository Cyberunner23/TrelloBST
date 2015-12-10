

use std::fs::File;
use std::io::Read;
use std::path::Path;
use std::process::exit;


extern crate clap;
use clap::{Arg, App};

extern crate hyper;
use hyper::Client;


mod appveyor;
mod config;
mod travis_ci;
mod trello;


fn get_config_path(is_using_custom_config: &mut bool, path_str: &str) -> Result<config::TrelloBSTConfigPath, &'static str> {

    if *is_using_custom_config {
        println!("Looking for the configuration file: {}", path_str);
        let path = Path::new(path_str);
        match config::TrelloBSTConfigPath::try_custom_config_path(path) {
            Ok(_config) => {
                return Ok(_config)
            },
            Err(c) => {
                return Err(c)
            },
        };
    } else {
        println!("Looking for the configuration file in default location...");
        match config::TrelloBSTConfigPath::try_default_config_path() {
            Ok(_config) => {
                return Ok(_config)
            },
            Err(c) => {
                return Err(c)
            },
        };
    }
}

fn parse_config(config_path: &config::TrelloBSTConfigPath) -> Result<config::TrelloBSTAPIConfig, &'static str>{
    match File::open(config_path.config_path.as_path()) {
        Ok(file) => {
            let mut config_file = file;
            match config::TrelloBSTAPIConfig::from_file(&mut config_file) {
                Ok(_config) => {
                    return Ok(_config)
                }
                Err(err)   => {
                    return Err(err)
                }
            }
        }
        Err(_)   =>{
            return Err("Cannot open config file for parsing, configuration file won't be used...");
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

    let mut config_path = config::TrelloBSTConfigPath::new();

    if is_using_config_file {
        match get_config_path(&mut is_using_custom_config, matches.value_of("CONFIG").unwrap_or("")) {
            Ok(config) => {
                config_path = config;
                println!("Found.")
            }
            Err(err)=>{
                println!("An error occurred: {}", err);
                println!("Configuration file won't be used...");
                is_using_config_file = false;
            }
        }
    }


    ////////////////////////////////////////////////////////////
    //                     Parse config                       //
    ////////////////////////////////////////////////////////////

    let mut config = config::TrelloBSTAPIConfig::new();

    if is_using_config_file {
        println!("Parsing...");
        match parse_config(&config_path) {
            Ok(_config) => {
                config = _config;
                println!("Done.");
            }
            Err(err)   => {
                println!("{}", err);
                is_using_config_file = false;
            }
        }
    }


    ////////////////////////////////////////////////////////////
    //                   Setup Trello API                     //
    ////////////////////////////////////////////////////////////

    trello::setup_api(&mut config);

    if is_using_config_file{
        match config::TrelloBSTAPIConfig::save_config(&config_path, &config) {
            Ok(_)    => (),
            Err(err) => println!("{}", err)
        }
    }


    ////////////////////////////////////////////////////////////
    //                  Setup Trello Board                    //
    ////////////////////////////////////////////////////////////

    let mut board_info = trello::TrelloBoardInfo::new();

    //trello::setup_board(&config, &mut board_info);

    let     http_client = Client::new();
    let mut res         = http_client.get("https://api.trello.com/1/members/me?fields=&boards=all&board_fields=name&key=0e190833c4db5fd7d3b0b26ae642d6fa&token=172cb3e72e91da9c43a0524f3bc4b8aaaf7091d3a47aeb8c4f464744560a188d").send().unwrap();

    let mut body = String::new();
    res.read_to_string(&mut body).unwrap();
    println!("Response: {}\n\n\n", body);


    //let test: trello::members_me_boards_response = serde_json::to_string(&body).unwrap();


}


