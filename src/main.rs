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
use std::fs::Metadata;
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::process::exit;


extern crate clap;
use clap::{Arg, App};

extern crate hyper;

extern crate serde;
extern crate serde_json;

extern crate term;


mod appveyor;
mod config;
mod travis_ci;
mod trello;
mod utils;


//TODO: Major cleanup!!!
//TODO: Macroize common routines.
//TODO: Put common routines in util file.

include!("utils_macros.rs");


fn get_ci_config_output_dir(term: &mut Box<term::StdoutTerminal>) -> PathBuf {

    //Get current working directory.
    let mut current_working_dir: PathBuf    = PathBuf::new();
    let mut get_current_working_dir_errored = false;
    match env::current_dir() {
        Ok(path_buf) => {
            current_working_dir = path_buf;
        },
        Err(err) => {
            writeln_red!(term, "An error occurred while getting the current working directory: {}", err.description());
            get_current_working_dir_errored = true;
        }
    }

    //Get input
    let mut option_string = String::new();
    loop{

        if get_current_working_dir_errored {
            print!("Please enter the directory in which you want the configuration file to be outputted:");
        } else {
            print!("Please enter the directory in which you want the configuration file to be outputted. [The default path is the current working directory: {} ]: ", current_working_dir.to_str().unwrap());
        }

        match_to_none!(term.flush());

        loop {

            match io::stdin().read_line(&mut option_string) {
                Ok(_)  => option_string = option_string.trim_matches('\n').to_string(),
                Err(_) => {panic!("Error while reading the input.");}
            }

            if option_string.is_empty() {
                if get_current_working_dir_errored {
                    writeln_red!(term, "Please enter a path.");
                } else {
                    option_string = current_working_dir.to_str().unwrap().to_string();
                    break;
                }
            } else {
                break;
            }
        }

        let mut dir_metadata: Metadata;
        match fs::metadata(PathBuf::from(&option_string)) {
            Ok(metadata) => {
                if metadata.is_dir() {
                    break;
                } else {
                    writeln_red!(term, "Error: The path provided is not a valid path.");
                }
            },
            Err(_)       => {
                writeln_red!(term, "Error: Failed to acquire directory's metadata, please enter a valid path.");
            }
        }

    }

    PathBuf::from(option_string)
}


fn main() {

    let     trellobst_version = "0.0.1";
    let mut term              = term::stdout().unwrap();

    writeln_green!(term, "╔══════════════════════════════════════════════════════════╗");
    writeln_green!(term, "║         Welcome to the Trello Build Status Tool.         ║");
    writeln_green!(term, "╚══════════════════════════════════════════════════════════╝");


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
        writeln_red!(term, "Error: --config (-c) and --no-config (-n) cannot be used at the same time.");
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

    match config::TrelloBSTConfigPath::get_config_dir(&mut term, &mut is_using_config_file, &is_using_custom_config, matches.value_of("CONFIG").unwrap_or("")) {
        Ok(_config_path) => {
            config_path = _config_path;
        }
        Err(err)         => {
            is_using_config_file = false;
        }
    }

    //if is_using_custom_config {
    //    let path_str = matches.value_of("CONFIG").unwrap_or("");
    //    let path     = Path::new(path_str);
    //    let status   = utils::StatusPrint::from_string(&mut term, format!("Looking for the configuration file: {}", path_str));
    //    match config::TrelloBSTConfigPath::try_custom_config_path(path) {
    //        Ok(_config) => {
    //            config_path = _config;
    //            status.success(&mut term);
    //        },
    //        Err(err)    => {
    //            is_using_config_file = false;
    //            status.error(&mut term);
    //            writeln_red!(term, "An error occurred: {}", err);
    //            writeln_red!(term, "Configuration file won't be used...");
    //        },
    //    };
    //} else {
    //    let status = utils::StatusPrint::from_str(&mut term, "Looking for the configuration file in default location...");
    //    match config::TrelloBSTConfigPath::try_default_config_path() {
    //        Ok(_config) => {
    //            status.success(&mut term);
    //            config_path = _config;
    //        },
    //        Err(err)    => {
    //            is_using_config_file = false;
    //            status.error(&mut term);
    //            term.fg(term::color::RED).unwrap();
    //            match_to_none!(writeln!(term, "An error occurred: {}", err));
    //            match_to_none!(writeln!(term, "Configuration file won't be used..."));
    //            term.reset().unwrap();
    //        },
    //    };
    //}


    ////////////////////////////////////////////////////////////
    //                     Parse config                       //
    ////////////////////////////////////////////////////////////

    let mut config = config::TrelloBSTAPIConfig::new();

    if is_using_config_file {
        let status = utils::StatusPrint::from_str(&mut term, "Parsing the configuration file.");
        match config::TrelloBSTAPIConfig::from_file(&config_path) {
            Ok(_config) => {
                config = _config;
                status.success(&mut term);
            }
            Err(err)    => {
                status.error(&mut term);
                writeln_red!(term, "An error occurred: {}", err);
                writeln_red!(term, "Configuration file won't be used...");
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
            Err(err) => {
                writeln_red!(term, "Error: {}", err);
                writeln_red!(term, "Configuration file won't be used...");
                is_using_config_file = false;
            }
        }
    }


    ////////////////////////////////////////////////////////////
    //                  Setup Trello Board                    //
    ////////////////////////////////////////////////////////////

    let mut board_info = trello::TrelloBoardInfo::new();
    let mut board_list = trello::MembersMeBoardsResponse::new();


    //Select/create board
    let status = utils::StatusPrint::from_str(&mut term, "Acquiring board list from Trello.");
    match trello::acquire_board_list(&config, &mut board_list) {
        Ok(_)    => {
            status.success(&mut term);
        },
        Err(err) => {
            status.error(&mut term);
            panic!(format!("An error occurred while communicating with Trello: {}", err));
        },
    }

    println!("Which board do you want to setup?");
    let mut counter = 1;
    for i in 0..board_list.boards.len() {
        println!("[{}] {}", i + 1, board_list.boards[i].name);
        counter += 1;
    }
    writeln_green!(term, "[{}] Create a new board.", counter);

    let mut option_str    = String::new();
    let mut option: usize = 0;
    loop {
        print!("Please enter an option: ");
        match_to_none!(term.flush());
        match io::stdin().read_line(&mut option_str) {
            Ok(_)  => {
                option_str = option_str.trim_matches('\n').to_string();
                match option_str.parse::<usize>(){
                    Ok(_option) => {
                        option = _option;
                    },
                    Err(_)      => {
                        option_str.clear();
                        writeln_red!(term, "Error while parsing the input.");
                    }
                }
            },
            Err(_) => {panic!("Error while reading the input.");}
        }

        if option <= counter && option > 0 {
            break;
        }else {
            option_str.clear();
            writeln_red!(term, "Please enter a valid option.");
        }
    }

    let mut is_board_created = false;
    if option == counter {
        match trello::create_board_and_list(&mut term, &config, &mut board_info){
            Ok(_)    => is_board_created = true,
            Err(err) => {
                panic!(format!("An error occured: {}", err));
            }
        }
    } else {
        board_info.board_id = board_list.boards[option - 1].id.clone();
    }


    //Select/create board list
    if !is_board_created {
        let status = utils::StatusPrint::from_str(&mut term, "Acquiring board's lists list from Trello.");
        let mut board_lists_list = trello::BoardsResponse::new();
        match trello::acquire_board_lists_list(&config, &board_info, &mut board_lists_list) {
            Ok(_)    => {
                status.success(&mut term);
            },
            Err(err) => {
                status.error(&mut term);
                panic!(format!("An error occurred while communicating with Trello: {}", err));
            },
        }

        println!("Which board list do you want to use for the build statuses?");

        let mut counter = 1;
        for i in 0..board_lists_list.lists.len() {
            println!("[{}] {}", i + 1, board_lists_list.lists[i].name);
            counter += 1;
        }
        writeln_red!(term, "[{}] Create a new list.", counter);


        let mut option_str    = String::new();
        let mut option: usize = 0;
        loop {
            print!("Please enter an option: ");
            match_to_none!(term.flush());
            match io::stdin().read_line(&mut option_str) {
                Ok(_)  => {
                    option_str = option_str.trim_matches('\n').to_string();
                    match option_str.parse::<usize>(){
                        Ok(_option) => {
                            option = _option;
                        },
                        Err(_)      => {
                            option_str.clear();
                            writeln_red!(term, "Error while parsing the input.");
                        }
                    }
                },
                Err(_) => {panic!("Error while reading the input.");}
            }

            if option <= counter && option > 0 {
                break;
            }else {
                option_str.clear();
                writeln_red!(term, "Please enter a valid option.");
            }
        }

        if option == counter {
            match trello::create_list(&mut term, &config, &mut board_info){
                Ok(_)    => (),
                Err(err) => {
                    panic!(format!("An error occured: {}", err));
                }
            }
        } else {
            board_info.list_id = board_lists_list.lists[option - 1].id.clone();
        }
    }

    //TODO: Different behavior on fail?
    //create labels
    match trello::create_pass_fail_labels(&config, &mut board_info){
        Ok(_) => (),
        Err(err) => {writeln_red!(term, "Error creating the labels: {}", err);}
    }


    ////////////////////////////////////////////////////////////
    //               Setup Travis-CI/Appveyor                 //
    ////////////////////////////////////////////////////////////

    loop {

        //Print options
        println!("For which continuous integration service do you want a configuration file for?");
        println!("[1] Travis-CI");
        println!("[2] AppVeyor");
        writeln_red!(term, "[3] Quit.");

        let mut option_str    = String::new();
        let mut option: usize = 0;
        loop {
            print!("Please enter an option: ");
            match_to_none!(term.flush());
            match io::stdin().read_line(&mut option_str) {
                Ok(_)  => {
                    option_str = option_str.trim_matches('\n').to_string();
                    match option_str.parse::<usize>(){
                        Ok(_option) => {
                            option = _option;
                        },
                        Err(_)      => {
                            option_str.clear();
                            writeln_red!(term, "Error while parsing the input.");
                        }
                    }
                },
                Err(_) => {panic!("Error while reading the input.");}
            }

            if option <= 3 && option > 0 {
                break;
            }else {
                option_str.clear();
                writeln_red!(term, "Please enter a valid option.");
            }
        }

        //Get Travis-CI/Appveyor config file output dir.
        let mut ci_config_output_dir = get_ci_config_output_dir(&mut term);

        //TODO: Major cleanup, this is a mess....
        match option {
            1 => {
                let mut travis_yml_path     = ci_config_output_dir;
                let mut is_api_setup_failed = false;
                let mut is_file_create_fail = false;
                travis_yml_path.push(".travis.yml");

                //Get access token / API key
                //NOTE: A little workaround... Apparently cannot check if a borrowed bool is true...
                //TODO: use *bool to do compare
                let status   = utils::StatusPrint::from_str(&mut term, "Setting up the Travis-CI API key.");
                match travis_ci::setup_api(&mut term, is_using_config_file, &mut config){
                    Ok(_is_using_config_file) => {
                        is_using_config_file = _is_using_config_file;
                        if is_using_config_file {
                            match config::TrelloBSTAPIConfig::save_config(&config_path, &config) {
                                Ok(_)    => (),
                                Err(err) => {
                                    writeln_red!(term, "Error: {}", err);
                                    writeln_red!(term, "Configuration file won't be used...");
                                    is_using_config_file = false;
                                }
                            }
                        }
                        status.success(&mut term);
                    }
                    Err(err)                  => {
                        status.error(&mut term);
                        writeln_red!(term, "Error setting up the travis-CI API key: {}", err);
                        is_api_setup_failed = true;
                    }
                }

                //Get repo tag
                loop{

                    //Get repo tag
                    print!("Please enter the repo you wish to get the .travis.yml for in the form of user/repo: ");
                    match_to_none!(term.flush());
                    option_str.clear();
                    match io::stdin().read_line(&mut option_str) {
                        Ok(_)  => {
                            option_str = option_str.trim_matches('\n').to_string();
                            match option_str.parse::<usize>(){
                                Ok(_option) => {
                                    option = _option;
                                },
                                Err(_)      => {
                                    option_str.clear();
                                    writeln_red!(term, "Error while parsing the input.");
                                    is_file_create_fail = true;
                                }
                            }
                        },
                        Err(_) => {panic!("Error while reading the input.");}
                    }

                    if !is_file_create_fail {
                        //TODO: Create .travis.yml
                        //if invalid repo tag, retry, if anything else loop around to ci select
                    }
                }

                if !is_api_setup_failed || !is_file_create_fail{
                    break;
                }
            },
            2 => {
                let mut appveyor_yml_path = ci_config_output_dir;
                appveyor_yml_path.push("appveyor.yml");
                //TODO: Setup AppVeyor API Key
                //TODO: Create appveyor.yml

                break;
            },
            3 => exit(0),
            _ => {panic!("An invalid option slipped through...");}
        }
    }
}


