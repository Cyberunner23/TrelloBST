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
use std::fs::{File, Metadata};
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


//TODO: Major cleanup!!!
//TODO: Macroize common routines.
//TODO: Put common routines in util file.


macro_rules! match_to_none {
    ($match_expr:expr) => {
        match $match_expr {
            Ok(_)  => (),
            Err(_) => (),
        }
    }
}

macro_rules! status_print {
    ($term:expr, $($msg:tt)*) => {
        match_to_none!($term.write_fmt(format_args!("[ ] {}", format_args!($($msg)*))));
        match_to_none!($term.flush());
    }
}

macro_rules! status_print_success {
    ($term:expr, $($msg:tt)*) => {
        match_to_none!($term.carriage_return());
        match_to_none!($term.write_fmt(format_args!("[")));
        match_to_none!($term.fg(term::color::GREEN));
        match_to_none!($term.write_fmt(format_args!("✓")));
        match_to_none!($term.reset());
        match_to_none!($term.write_fmt(format_args!("] {}\n", format_args!($($msg)*))));
        match_to_none!($term.flush());
    }
}

macro_rules! status_print_error {
    ($term:expr, $($msg:tt)*) => {
        match_to_none!($term.carriage_return());
        match_to_none!($term.write_fmt(format_args!("[")));
        match_to_none!($term.fg(term::color::RED));
        match_to_none!($term.write_fmt(format_args!("✗")));
        match_to_none!($term.reset());
        match_to_none!($term.write_fmt(format_args!("] {}\n", format_args!($($msg)*))));
        match_to_none!($term.flush());
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
                Err(err)    => {
                    return Err(err)
                }
            }
        }
        Err(_)   =>{
            return Err("Cannot open config file for parsing, configuration file won't be used...");
        }
    }
}


fn get_ci_config_output_dir(term: &mut Box<term::StdoutTerminal>) -> PathBuf {

    //Get current working directory.
    let mut current_working_dir: PathBuf    = PathBuf::new();
    let mut get_current_working_dir_errored = false;
    match env::current_dir() {
        Ok(path_buf) => {
            current_working_dir = path_buf;
        },
        Err(err) => {
            term.fg(term::color::RED).unwrap();
            writeln!(term, "An error occurred while getting the current working directory: {}", err.description()).unwrap();
            term.reset().unwrap();
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
                    term.fg(term::color::RED).unwrap();
                    writeln!(term, "Please enter a path.");
                    term.reset().unwrap();
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
                    term.fg(term::color::RED).unwrap();
                    writeln!(term, "Error: The path provided is not a valid path.");
                    term.reset().unwrap();
                }
            },
            Err(_)       => {
                term.fg(term::color::RED).unwrap();
                writeln!(term, "Error: Failed to acquire directory's metadata, please enter a valid path.");
                term.reset().unwrap();
            }
        }

    }

    PathBuf::from(option_string)
}


fn main() {

    let     trellobst_version = "0.0.1";
    let mut term              = term::stdout().unwrap();

    term.fg(term::color::GREEN).unwrap();
    writeln!(term, "╔══════════════════════════════════════════════════════════╗").unwrap();
    writeln!(term, "║         Welcome to the Trello Build Status Tool.         ║").unwrap();
    writeln!(term, "╚══════════════════════════════════════════════════════════╝").unwrap();
    term.reset().unwrap();


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
        term.fg(term::color::RED).unwrap();
        match_to_none!(writeln!(term, "Error: --config (-c) and --no-config (-n) cannot be used at the same time."));
        term.reset().unwrap();
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

    if is_using_custom_config {
        let path_str = matches.value_of("CONFIG").unwrap_or("");
        status_print!(term, "Looking for the configuration file: {}", path_str);
        let path = Path::new(path_str);
        match config::TrelloBSTConfigPath::try_custom_config_path(path) {
            Ok(_config) => {
                config_path = _config;
                status_print_success!(term, "Looking for the configuration file: {}", path_str);
            },
            Err(err)    => {
                is_using_config_file = false;
                status_print_error!(term, "Looking for the configuration file: {}", path_str);
                term.fg(term::color::RED).unwrap();
                match_to_none!(writeln!(term, "An error occurred: {}", err));
                match_to_none!(writeln!(term, "Configuration file won't be used..."));
                term.reset().unwrap();
            },
        };
    } else {
        status_print!(term, "Looking for the configuration file in default location...");
        match config::TrelloBSTConfigPath::try_default_config_path() {
            Ok(_config) => {
                status_print_success!(term, "Looking for the configuration file in default location...");
                config_path = _config;
            },
            Err(err)    => {
                is_using_config_file = false;
                status_print_error!(term, "Looking for the configuration file in default location...");
                term.fg(term::color::RED).unwrap();
                match_to_none!(writeln!(term, "An error occurred: {}", err));
                match_to_none!(writeln!(term, "Configuration file won't be used..."));
                term.reset().unwrap();
            },
        };
    }


    ////////////////////////////////////////////////////////////
    //                     Parse config                       //
    ////////////////////////////////////////////////////////////

    let mut config = config::TrelloBSTAPIConfig::new();

    if is_using_config_file {
        status_print!(term, "Parsing the configuration file.");
        match parse_config(&config_path) {
            Ok(_config) => {
                config = _config;
                status_print_success!(term, "Parsing the configuration file.");
            }
            Err(err)    => {
                status_print_error!(term, "Parsing the configuration file.");
                term.fg(term::color::RED).unwrap();
                match_to_none!(writeln!(term, "An error occurred: {}", err));
                match_to_none!(writeln!(term, "Configuration file won't be used..."));
                term.reset().unwrap();
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
                term.fg(term::color::RED).unwrap();
                match_to_none!(writeln!(term, "Error: {}", err));
                match_to_none!(writeln!(term, "Configuration file won't be used..."));
                term.reset().unwrap();
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
    status_print!(term, "Acquiring board list from Trello.");
    match trello::acquire_board_list(&config, &mut board_list) {
        Ok(_)    => {
            status_print_success!(term, "Acquiring board list from Trello.");
        },
        Err(err) => {
            status_print_error!(term, "Acquiring board list from Trello.");
            panic!(format!("An error occurred while communicating with Trello: {}", err));
        },
    }

    println!("Which board do you want to setup?");
    let mut counter = 1;
    for i in 0..board_list.boards.len() {
        println!("[{}] {}", i + 1, board_list.boards[i].name);
        counter += 1;
    }
    term.fg(term::color::GREEN).unwrap();
    println!("[{}] Create a new board.", counter);
    term.reset().unwrap();

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
                        term.fg(term::color::RED).unwrap();
                        match_to_none!(writeln!(term, "Error while parsing the input."));
                        term.reset().unwrap();
                    }
                }
            },
            Err(_) => {panic!("Error while reading the input.");}
        }

        if option <= counter && option > 0 {
            break;
        }else {
            option_str.clear();
            term.fg(term::color::RED).unwrap();
            match_to_none!(writeln!(term, "Please enter a valid option."));
            term.reset().unwrap();
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
        status_print!(term, "Acquiring board's lists list from Trello.");
        let mut board_lists_list = trello::BoardsResponse::new();
        match trello::acquire_board_lists_list(&config, &board_info, &mut board_lists_list) {
            Ok(_)    => {
                status_print_success!(term, "Acquiring board's lists list from Trello.");
            },
            Err(err) => {
                status_print_error!(term, "Acquiring board's lists list from Trello.");
                panic!(format!("An error occurred while communicating with Trello: {}", err));
            },
        }

        println!("Which board list do you want to use for the build statuses?");

        let mut counter = 1;
        for i in 0..board_lists_list.lists.len() {
            println!("[{}] {}", i + 1, board_lists_list.lists[i].name);
            counter += 1;
        }
        term.fg(term::color::GREEN).unwrap();
        writeln!(term, "[{}] Create a new list.", counter);
        term.reset().unwrap();

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
                            term.fg(term::color::RED).unwrap();
                            match_to_none!(writeln!(term, "Error while parsing the input."));
                            term.reset().unwrap();
                        }
                    }
                },
                Err(_) => {panic!("Error while reading the input.");}
            }

            if option <= counter && option > 0 {
                break;
            }else {
                option_str.clear();
                term.fg(term::color::RED).unwrap();
                match_to_none!(writeln!(term, "Please enter a valid option."));
                term.reset().unwrap();
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

    //create labels

    match trello::create_pass_fail_labels(&config, &mut board_info){
        Ok(_) => (),
        Err(err) => {println!("{}", format!("Error creating the labels: {}", err));}
    }


    ////////////////////////////////////////////////////////////
    //               Setup Travis-CI/Appveyor                 //
    ////////////////////////////////////////////////////////////

    loop {

        //Print options
        println!("For which continuous integration service do you want a configuration file for?");
        println!("[1] Travis-CI");
        println!("[2] AppVeyor");
        term.fg(term::color::RED).unwrap();
        writeln!(term, "[3] Quit.");
        term.reset().unwrap();

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
                            term.fg(term::color::RED).unwrap();
                            match_to_none!(writeln!(term, "Error while parsing the input."));
                            term.reset().unwrap();
                        }
                    }
                },
                Err(_) => {panic!("Error while reading the input.");}
            }

            if option <= 3 && option > 0 {
                break;
            }else {
                option_str.clear();
                term.fg(term::color::RED).unwrap();
                match_to_none!(writeln!(term, "Please enter a valid option."));
                term.reset().unwrap();
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
                status_print!(term, "Setting up the Travis-CI API key.");
                match travis_ci::setup_api(&mut term, is_using_config_file, &mut config){
                    Ok(_is_using_config_file) => {
                        is_using_config_file = _is_using_config_file;
                        if is_using_config_file {
                            match config::TrelloBSTAPIConfig::save_config(&config_path, &config) {
                                Ok(_)    => (),
                                Err(err) => {
                                    term.fg(term::color::RED).unwrap();
                                    match_to_none!(writeln!(term, "Error: {}", err));
                                    match_to_none!(writeln!(term, "Configuration file won't be used..."));
                                    term.reset().unwrap();
                                    is_using_config_file = false;
                                }
                            }
                        }
                        status_print_success!(term, "Setting up the Travis-CI API key.");
                    }
                    Err(err)                  => {
                        status_print_error!(term, "Setting up the Travis-CI API key.");
                        term.fg(term::color::RED).unwrap();
                        match_to_none!(writeln!(term, "Error setting up the travis-CI API key: {}", err));
                        term.reset().unwrap();
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
                                    term.fg(term::color::RED).unwrap();
                                    match_to_none!(writeln!(term, "Error while parsing the input."));
                                    term.reset().unwrap();
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


