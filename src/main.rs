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
mod push;


////////////////////////////////////////////////////////////
//                         Macros                         //
////////////////////////////////////////////////////////////

include!("utils_macros.rs");


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
            err_string.push_str("\", do you have permission to read/write to this directory?");
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
                    let mut err_string = "Invalid directory: \"".to_string();
                    err_string.push_str(err.description());
                    err_string.push_str("\"");
                    return Err(err_string);
                }
            }
        }
        counter += 1;
    }
}


////////////////////////////////////////////////////////////
//                          Main                          //
////////////////////////////////////////////////////////////

fn main() {

    let     trellobst_version = "2.0.0-dev";
    //NOTE: Public developer key
    let     trellobst_api_key = "0e190833c4db5fd7d3b0b26ae642d6fa";
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
        .subcommand(SubCommand::with_name("push")
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
                 .required(false))
            .arg(Arg::with_name("BUILD_PASS")
                 .conflicts_with("BUILD_FAIL")
                 .short("p")
                 .long("pass")
                 .help("Sets build status to passed.")
                 .takes_value(false))
            .arg(Arg::with_name("BUILD_FAIL")
                 .conflicts_with("BUILD_PASS")
                 .short("f")
                 .long("fail")
                 .help("Sets build status to failed.")
                 .takes_value(false))
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
            .conflicts_with("PRINT_OUTPUT")
            .short("o")
            .long("output")
            .help("Sets the output directory for the CI configuration file.")
            .validator(dir_path_validator)
            .takes_value(true))
        .arg(Arg::with_name("PRINT_OUTPUT")
            .conflicts_with("OUTPUT_DIR")
            .short("p")
            .long("print")
            .help("Print the resulting CI config instead of putting it in a file.")
            .takes_value(false))
        .get_matches();


    if let Some(push_matches) = matches.subcommand_matches("push") {

        //Validate --pass --fail
        if !push_matches.is_present("BUILD_PASS") && !push_matches.is_present("BUILD_FAIL") {
            writeln_red!(term, "Error: One of --pass and --fail must be used.");
            exit(-1);
        }

        //Create config struct
        let status = utils::StatusPrint::from_str(&mut term, "Pushing card to Trello.");
        let push_config: push::PushConfig;
        match push::PushConfig::fill(push_matches.value_of("CARD_TITLE").unwrap().to_string(),
                                     push_matches.value_of("CARD_DESC").unwrap_or("").to_string(),
                                     push_matches.value_of("TRELLO_BUILD_PASS_ID").unwrap_or("").to_string(),
                                     push_matches.value_of("TRELLO_BUILD_FAIL_ID").unwrap_or("").to_string(),
                                     push_matches.value_of("TRELLO_LIST_ID").unwrap_or("").to_string(),
                                     push_matches.value_of("TRELLO_API_TOKEN").unwrap_or("").to_string()) {
            Ok(_push_config) => {push_config = _push_config;}
            Err(err)         => {
                status.error(&mut term);
                writeln_red!(term, "{}", err);
                exit(-1);
            }
        }

        //Push card to Trello
        match push::push(trellobst_api_key.to_string(), push_matches.is_present("BUILD_PASS"), push_config) {
            Ok(()) => {
                status.success(&mut term);
                exit(0);
            }
            Err(err) => {
                status.error(&mut term);
                writeln_red!(term, "Error while pushing the card to Trello: {}", err);
                exit(-1);
            }
        }
    }


    //If push subcommand not used i.e. generate a CI config
    let mut config_mode: Option<PathBuf>;
    let mut output_mode: Option<PathBuf>;

    //Check config file cli options
    if !matches.is_present("CONFIG") && !matches.is_present("NO_CONFIG") {

        //Default config
        //Check if home directory works
        let valid_path_found: bool;
        match env::home_dir() {
            Some(home_dir) => {
                let mut config_file_path = home_dir;
                config_file_path.push(".TrelloBST.cfg");
                match file_path_validator(config_file_path.to_str().unwrap_or("~/.TrelloBST.cfg").to_string()) {
                    Ok(()) => {
                        println!("Config file location set to: {:?}", config_file_path);
                        config_mode = Option::Some(config_file_path);
                        valid_path_found = true;
                    }
                    Err(_) => {valid_path_found = false}
                }
            }
            None           => {
                writeln_red!(term, "Error: Failed to acquire the home directory path.");
                valid_path_found = false;
            }
        }

        if !valid_path_found {

            let config_file_path_str = "./.TrelloBST.cfg".to_string();
            let config_file_path = PathBuf::from(&config_file_path_str);

            writeln_red!(term, "Error: Failed to read/create the configuration file in the home directory. Falling back to ./.TrelloBST.cfg");
            match file_path_validator(config_file_path_str.clone()) {
                Ok(()) => {
                    config_mode = Option::Some(PathBuf::from(config_file_path));
                    println!("Config file location set to: {}", config_file_path_str);
                }
                Err(_) => {
                    config_mode = Option::None;
                    writeln_red!(term, "Error: Failed to read/create the configuration file at ./.TrelloBST.cfg. TrelloBST will continue without saving inputted values in the configuration file.");
                }
            }
        }

    } else if matches.is_present("CONFIG") {
        //Custom config
        println!("Config file location set to: {}", matches.value_of("CONFIG").unwrap());
        config_mode = Option::Some(PathBuf::from(matches.value_of("CONFIG").unwrap()));
    } else if matches.is_present("NO_CONFIG") {
        config_mode = Option::None;
    }


    //Check CI config output cli options
    if !matches.is_present("OUTPUT_DIR") && !matches.is_present("PRINT_OUTPUT") {

        //Output to current directory
        //Try current directory
        match dir_path_validator("./".to_string()) {
            Ok(()) => {
                output_mode = Option::Some(PathBuf::from("./".to_string()));
            }
            Err(_) => {
                writeln_red!(term, "Failed to acquire current directory, CI config will be printed in the terminal.");
                output_mode = Option::None;
            }
        }

    } else if matches.is_present("OUTPUT_DIR") {
        //Output to custom directory
        output_mode = Option::Some(PathBuf::from(matches.value_of("OUTPUT_DIR").unwrap()));
    } else if matches.is_present("PRINT_OUTPUT") {
        //Print the output
        output_mode = Option::None;
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
