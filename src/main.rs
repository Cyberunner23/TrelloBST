/*
    Copyright (c) 2015, Alex Frappier Lachapelle
    All rights reserved.

    Redistribution and use in source and binary forms, with or without
    modification, are permitted provided that the following conditions are met:

    1. Redistributions of source code must retain the above copyright notice, this
       list of conditions and the following disclaimer.
    2. Redistributions in binary form must reproduce the above copyright notice,
       this list of conditions and the following disclaimer in the documentation
       and/or other materials provided with the distribution.

    THIS SOFTWARE IS PROVIDED BY THE COPYRIGHT HOLDERS AND CONTRIBUTORS "AS IS" AND
    ANY EXPRESS OR IMPLIED WARRANTIES, INCLUDING, BUT NOT LIMITED TO, THE IMPLIED
    WARRANTIES OF MERCHANTABILITY AND FITNESS FOR A PARTICULAR PURPOSE ARE
    DISCLAIMED. IN NO EVENT SHALL THE COPYRIGHT OWNER OR CONTRIBUTORS BE LIABLE FOR
    ANY DIRECT, INDIRECT, INCIDENTAL, SPECIAL, EXEMPLARY, OR CONSEQUENTIAL DAMAGES
    (INCLUDING, BUT NOT LIMITED TO, PROCUREMENT OF SUBSTITUTE GOODS OR SERVICES;
    LOSS OF USE, DATA, OR PROFITS; OR BUSINESS INTERRUPTION) HOWEVER CAUSED AND
    ON ANY THEORY OF LIABILITY, WHETHER IN CONTRACT, STRICT LIABILITY, OR TORT
    (INCLUDING NEGLIGENCE OR OTHERWISE) ARISING IN ANY WAY OUT OF THE USE OF THIS
    SOFTWARE, EVEN IF ADVISED OF THE POSSIBILITY OF SUCH DAMAGE.
*/


use std::env;
use std::error::Error;
use std::fs;
use std::io::{self, Write};
use std::path::PathBuf;
use std::process::exit;

extern crate clap;
use clap::{Arg, App};

extern crate hyper;
use hyper::header::Headers;

extern crate serde;
extern crate serde_json;

extern crate term;

mod appveyor;
mod config;
mod travis_ci;
mod trello;
mod utils;


////////////////////////////////////////////////////////////
//                         Macros                         //
////////////////////////////////////////////////////////////

include!("utils_macros.rs");


fn main() {

    let     trellobst_version = "0.0.1";
    let mut term              = term::stdout().unwrap();

    writeln_green!(term, "╔══════════════════════════════════════════════════════════╗");
    writeln_green!(term, "║         Welcome to the Trello Build Status Tool.         ║");
    writeln_green!(term, "╚══════════════════════════════════════════════════════════╝");

    ////////////////////////////////////////////////////////////
    //               Parse command line options               //
    ////////////////////////////////////////////////////////////

    let mut config_file_path = PathBuf::new();
    let mut output_direcrory = PathBuf::new();

    let matches = App::new("TrelloBST")
        .version(trellobst_version)
        .arg(Arg::with_name("CONFIG")
            .short("c")
            .long("config")
            .help("Sets a custom TrelloBST configuration file.")
            .takes_value(true))
        .arg(Arg::with_name("NO-CONFIG")
            .short("n")
            .long("no-config")
            .help("Won't use a configuration file for TrelloBST.")
            .takes_value(false))
        .arg(Arg::with_name("OUTPUT_DIR")
            .short("o")
            .long("output")
            .help("Sets the output directory for the CI configuration file.")
            .takes_value(true))
        .get_matches();

    if matches.is_present("CONFIG") && matches.is_present("NO-CONFIG") {
        writeln_red!(term, "Error: --config (-c) and --no-config (-n) cannot be used at the same time.");
        exit(-1);
    }

    if matches.is_present("CONFIG") {
        config_file_path = PathBuf::from(matches.value_of("CONFIG").unwrap());
        let status = utils::StatusPrint::from_string(&mut term, format!("Reading/Creating the configuration file at {}", config_file_path.to_str().unwrap()));
        if !utils::is_valid_file_path(&mut term, &config_file_path) {
            status.error(&mut term);
            writeln_red!(term, "Error: Please enter a valid path for the output file. (Including read/write permissions.)");
            exit(-1);
        }
        status.success(&mut term);
    } else if !matches.is_present("NO-CONFIG") {

        let is_read_create_success: bool;
        let status = utils::StatusPrint::from_str(&mut term, "Reading/Creating the configuration file at the default location.");
        match env::home_dir() {
            Some(home_dir) => {
                config_file_path = home_dir;
                config_file_path.push(".TrelloBST.cfg");
                if utils::is_valid_file_path(&mut term, &config_file_path) {
                    status.success(&mut term);
                    is_read_create_success = true;
                } else {
                    status.error(&mut term);
                    is_read_create_success = false;
                }
            }
            None           => {
                status.error(&mut term);
                writeln_red!(term, "Error: Failed to acquire the home directory.");
                is_read_create_success = false;
            }
        }

        if !is_read_create_success {
            writeln_red!(term, "Error: Reading/Creating configuration file at default location failed. Falling back to ./.TrelloBST.cfg");
            let status = utils::StatusPrint::from_str(&mut term, "Reading/Creating the configuration file at ./.TrelloBST.cfg");
            config_file_path = PathBuf::from("./.TrelloBST.cfg".to_string());
            if utils::is_valid_file_path(&mut term, &config_file_path) {
                status.success(&mut term);
            } else {
                status.error(&mut term);
                writeln_red!(term, "Error: Reading/Creating configuration file at ./.TrelloBST.cfg failed.");
                exit(-2);
            }
        }
    }

    if matches.is_present("OUTPUT_DIR") {
        output_direcrory = PathBuf::from(matches.value_of("OUTPUT_DIR").unwrap());
        if !utils::is_valid_dir(&mut term, &output_direcrory) {
            writeln_red!(term, "Error: Please enter a valid path for the output directory. (Including read/write permissions.)");
            exit(-1);
        }
    } else {
        output_direcrory = PathBuf::from("./");
        if !utils::is_valid_dir(&mut term, &output_direcrory) {
            writeln_red!(term, "Error: Current directory is invalid. (Needs read/write permissions.)");
            exit(-1);
        }
    }

    ////////////////////////////////////////////////////////////
    //                     Parse config                       //
    ////////////////////////////////////////////////////////////

    let mut config = config::TrelloBSTAPIConfig::new();

    if config_file_path != PathBuf::new() {
        let status = utils::StatusPrint::from_str(&mut term, "Parsing the configuration file.");
        match config::TrelloBSTAPIConfig::from_file(&config_file_path) {
            Ok(_config) => {
                config = _config;
                status.success(&mut term);
            }
            Err(err)    => {
                status.error(&mut term);
                writeln_red!(term, "An error occurred: {}", err);
                writeln_red!(term, "Configuration file won't be used...");
                config_file_path = PathBuf::new()
            }
        }
    }


    ////////////////////////////////////////////////////////////
    //                   Setup Trello API                     //
    ////////////////////////////////////////////////////////////

    trello::setup_api(&mut term, &mut config);

    if config_file_path != PathBuf::new() {
        match config::TrelloBSTAPIConfig::save_config(&config_file_path, &config) {
            Ok(_)    => (),
            Err(err) => {
                writeln_red!(term, "Error: {}", err);
                writeln_red!(term, "Configuration file won't be used...");
                config_file_path = PathBuf::new()
            }
        }
    }


    ////////////////////////////////////////////////////////////
    //                  Setup Trello Board                    //
    ////////////////////////////////////////////////////////////

    let mut board_info = trello::TrelloBoardInfo::new();
    let mut board_list = trello::MembersMeBoardsResponse::new();

    let is_board_created = trello::setup_board(&mut term,  &mut config, &mut board_info);
                           trello::setup_list(&mut term,   &mut config, &mut board_info, &is_board_created);
                           trello::setup_labels(&mut term, &mut config, &mut board_info, &is_board_created);

    ////////////////////////////////////////////////////////////
    //               Setup Travis-CI/Appveyor                 //
    ////////////////////////////////////////////////////////////

    loop {

        //Print options
        println!("For which continuous integration service do you want a configuration file for?");
        println!("[1] Travis-CI");
        println!("[2] AppVeyor");
        writeln_red!(term, "[0] Quit.");

        let mut option: usize = 0;
        loop {
            get_input_usize!(term, &mut option, "Please enter an option: ");
            if option <= 3 && option >= 0 {
                break;
            }else {
                writeln_red!(term, "Please enter a valid option.");
            }
        }

        match option {
            1 => {

                //Get access token / API key
                match travis_ci::setup_api(&mut term, &mut config_file_path, &mut config) {
                    Ok(_)    => (),
                    Err(err) => {
                        writeln_red!(term, "Error setting up the travis-CI API token: {}", err);
                    }
                }

                //Save access token.
                if config_file_path != PathBuf::new() {
                    match config::TrelloBSTAPIConfig::save_config(&config_file_path, &config) {
                        Ok(_)    => (),
                        Err(err) => {
                            writeln_red!(term, "Error: {}", err);
                            writeln_red!(term, "Configuration file won't be used...");
                            config_file_path = PathBuf::new();
                        }
                    }
                }

                match travis_ci::create_travis_yml(&mut term, &config, &mut board_info, &mut output_direcrory) {
                    Ok(())   => (),
                    Err(err) => {
                        writeln_red!(term, "Error {}", err);
                    }
                }
            },
            2 => {

                //Get access token / API key
                appveyor::setup_api(&mut term, &mut config_file_path, &mut config);

                //Save access token.
                if config_file_path != PathBuf::new() {
                    match config::TrelloBSTAPIConfig::save_config(&config_file_path, &config) {
                        Ok(_)    => (),
                        Err(err) => {
                            writeln_red!(term, "Error: {}", err);
                            writeln_red!(term, "Configuration file won't be used...");
                            config_file_path = PathBuf::new();
                        }
                    }
                }

                //Create appveyor.yml
                match appveyor::create_appveyor_yml(&mut term, &config, &mut board_info, &mut output_direcrory) {
                    Ok(()) => (),
                    Err(err) => {writeln_red!(term, "Error {}", err);}
                }
            },
            0 => exit(0),
            _ => {panic!("An invalid option slipped through...");}
        }
    }
}
