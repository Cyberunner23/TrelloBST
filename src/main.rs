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
use std::fs::OpenOptions;
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::process::exit;

extern crate clap;
use clap::{Arg, App, AppSettings, SubCommand};

extern crate hyper;

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


////////////////////////////////////////////////////////////
//                      Structs/Enums                     //
////////////////////////////////////////////////////////////

pub enum RuntimeMode {
    Generate,
    Push
}

pub enum ConfigMode {
    None,
    Default,
    Custom
}

pub enum OutputMode {
    Default,
    Custom
}

pub struct GenConfig {
    pub config_mode:      ConfigMode,
    pub output_mode:      OutputMode,
    pub config_file_path: PathBuf,
    pub output_direcrory: PathBuf
}

pub struct PushConfig {
    pub cli_op_trello_api_token:         String,
    pub cli_op_trello_api_list_id:       String,
    pub cli_op_trello_api_build_pass_id: String,
    pub cli_op_trello_api_build_fail_id: String,
    pub card_title:                      String,
    pub card_desc:                       String,
    pub compress_dir:                    String
}


////////////////////////////////////////////////////////////
//                          Impls                         //
////////////////////////////////////////////////////////////

impl RuntimeMode {
    pub fn new(is_push_mode: &bool) -> RuntimeMode {
        if *is_push_mode {
            RuntimeMode::Push
        } else {
            RuntimeMode::Generate
        }
    }
}


////////////////////////////////////////////////////////////
//                          Funcs                         //
////////////////////////////////////////////////////////////

pub fn file_path_validator(file_path: String) -> Result<(), String> {
    match OpenOptions::new().read(true).write(true).create(true).open(Path::new(&file_path)) {
        Ok(_)  => Ok(()),
        Err(err) => {
            let mut err_string = "Cannot open file \"".to_string();
            err_string.push_str(file_path.as_str());
            err_string.push_str("\" due to an error: \"");
            err_string.push_str(err.description());
            err_string.push_str("\"");
            Err(err_string)
        }
    }
}

pub fn dir_path_validator(dir_path: String) -> Result<(), String> {

    //Check if it's even a directory.
    match fs::metadata(&dir_path) {
        Ok(metadata) => {
            if !metadata.is_dir() {
                let mut err_string = "\"".to_string();
                err_string.push_str(dir_path.as_str());
                err_string.push_str("\" is not a directory.");
                return Err(err_string);
            }
        },
        Err(err)     => {
            let mut err_string = "Failed to acquire metadata for \"".to_string();
            err_string.push_str(dir_path.as_str());
            err_string.push_str("\", do you have permission to write to this directory?");
            return Err(err_string);
        }
    }

    //Test if we can write a file to this directory
    let mut counter = 0;
    loop {
        //Generate a file name.
        let mut tmp_file_path:     String = dir_path.clone();
        let     tmp_file_name:     String = format!("ab{}ba.tmp", counter);
        let mut tmp_file_path_buf: PathBuf;
        tmp_file_path.push_str(tmp_file_name.as_str());
        tmp_file_path_buf = PathBuf::from(&tmp_file_path);

        //If file does not exist, check if we can create it with r/w permissions.
        if !tmp_file_path_buf.exists() {
            match OpenOptions::new().read(true).write(true).create(true).open(&tmp_file_path_buf.as_path()) {
                Ok(_)    => {
                    match fs::remove_file(&tmp_file_path_buf) {
                        Ok(_)  => return Ok(()),
                        Err(err) => {
                            let mut err_string = "Failed to delete temporary file: \"".to_string();
                            err_string.push_str(&tmp_file_path);
                            err_string.push_str("\"");
                            return Err(err_string);
                        }
                    }
                }
                Err(err) => {
                    let mut err_string = "Invalid output directory: \"".to_string();
                    err_string.push_str(err.description());
                    err_string.push_str("\"");
                    return Err(err_string);
                }
            }
        }
        counter += 1;
    }
}


fn main() {

    let     trellobst_version = "2.0.0-dev";
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
        .setting(AppSettings::SubcommandsNegateReqs)
        .subcommand(SubCommand::with_name("PUSH")
            .about("Pushes a build status to a trello board")
            .arg(Arg::with_name("CARD_TITLE")
                .short("t")
                .long("title")
                .help("Sets the title of the card.")
                .takes_value(true)
                .required(true))
            .arg(Arg::with_name("CARD_DESC")
                 .short("d")
                 .long("description")
                 .help("Sets the description of the card.")
                 .takes_value(true)
                 .required(true))
            .arg(Arg::with_name("TRELLO_API_TOKEN")
                 .short("T")
                 .long("token")
                 .help("Manually overrides the trello api token from the \"TRELLO_API_TOKEN\" environment variable.")
                 .takes_value(true)
                 .required(false))
            .arg(Arg::with_name("TRELLO_LIST_ID")
                 .short("L")
                 .long("list-id")
                 .help("Manually overrides the trello list id from the \"TRELLO_API_LIST_ID\" environment variable.")
                 .takes_value(true)
                 .required(false))
            .arg(Arg::with_name("TRELLO_BUILD_PASS_ID")
                 .short("P")
                 .long("pass-id")
                 .help("Manually overrides the trello build pass id from the \"TRELLO_API_BUILD_PASS_ID\" environment variable.")
                 .takes_value(true)
                 .required(false))
            .arg(Arg::with_name("TRELLO_BUILD_FAIL_ID")
                 .short("F")
                 .long("fail-id")
                 .help("Manually overrides the trello build fail id from the \"TRELLO_API_FAIL_PASS_ID\" environment variable.")
                 .takes_value(true)
                 .required(false))
        )
        .arg(Arg::with_name("CONFIG")
            .conflicts_with("NO-CONFIG")
            .short("c")
            .long("config")
            .help("Sets a custom TrelloBST configuration file.")
            .validator(file_path_validator)
            .takes_value(true))
        .arg(Arg::with_name("NO-CONFIG")
            .conflicts_with("CONFIG")
            .short("n")
            .long("no-config")
            .help("Won't use a configuration file for TrelloBST.")
            .takes_value(false))
        .arg(Arg::with_name("OUTPUT_DIR")
            .short("o")
            .long("output")
            .help("Sets the output directory for the CI configuration file.")
            .validator(dir_path_validator)
            .takes_value(true))
        .get_matches();


    let runtime_mode: RuntimeMode = RuntimeMode::new(&matches.is_present("PUSH"));
    match runtime_mode {
        RuntimeMode::Push => {
            //TODO: Push
        },
        RuntimeMode::Generate => {
            //TODO: Generate
        }
    }

    //----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------
    if matches.is_present("CONFIG") {
        config_file_path = PathBuf::from(matches.value_of("CONFIG").unwrap());
        let status = utils::StatusPrint::from_string(&mut term, format!("Reading/Creating the configuration file at {}", config_file_path.to_str().unwrap()));
        if !utils::is_valid_file_path(&config_file_path) {
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
                if utils::is_valid_file_path(&config_file_path) {
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
            if utils::is_valid_file_path(&config_file_path) {
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
            if option <= 3 {
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

                match travis_ci::create_travis_yml(&mut term, &config, &mut board_info, &output_direcrory) {
                    Ok(())   => (),
                    Err(err) => {
                        writeln_red!(term, "Error {}", err);
                    }
                }
            },
            2 => {

                //Get access token / API key
                appveyor::setup_api(&mut term, &mut config);

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
                match appveyor::create_appveyor_yml(&mut term, &config, &mut board_info, &output_direcrory) {
                    Ok(()) => (),
                    Err(err) => {writeln_red!(term, "Error {}", err);}
                }
            },
            0 => exit(0),
            _ => {panic!("An invalid option slipped through...");}
        }
    }
}
