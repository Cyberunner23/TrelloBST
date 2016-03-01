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
use std::fs::{File, Metadata, OpenOptions};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};

extern crate term;

mod utils;


////////////////////////////////////////////////////////////
//                         Macros                         //
////////////////////////////////////////////////////////////

include!("utils_macros.rs");


////////////////////////////////////////////////////////////
//                        Constants                       //
////////////////////////////////////////////////////////////
pub static trello_api_key: &'static str = "0e190833c4db5fd7d3b0b26ae642d6fa";


////////////////////////////////////////////////////////////
//                        Structs                         //
////////////////////////////////////////////////////////////


#[derive(Serialize, Deserialize)]
pub struct TrelloBSTAPIConfig {
    pub trello_app_token:    String,
    pub travis_access_token: String,
    pub appveyor_api_token:  String,
}


////////////////////////////////////////////////////////////
//                         Impls                          //
////////////////////////////////////////////////////////////

impl TrelloBSTAPIConfig {

    pub fn new() -> TrelloBSTAPIConfig {
        TrelloBSTAPIConfig {
            trello_app_token:    String::new(),
            travis_access_token: String::new(),
            appveyor_api_token:  String::new(),
        }
    }

    pub fn from_file(config_file_path: &PathBuf) -> Result<TrelloBSTAPIConfig, &'static str>{

        let mut file: File;
        match File::open(config_file_path.as_path()) {
            Ok(_file) => {
                file = _file;
            }
            Err(_)    =>{
                return Err("Cannot open config file for parsing, configuration file won't be used...");
            }
        }

        let metadata: Metadata;
        match file.metadata() {
            Ok(_metadata)  => {
                metadata = _metadata;
            }
            Err(_)         => {
                return Err("Cannot gather metadata of the configuration file, configuration file won't be used...")
            }
        }

        let api_config         = TrelloBSTAPIConfig::new();
        let file_length: usize = metadata.len() as usize;
        if file_length == 0 {
            return Ok(api_config)
        } else {
            let mut data: String = String::with_capacity(file_length +1);
            match file.read_to_string(&mut data) {
                Ok(_)  => {
                    //TODO: better error checking
                    return Ok(serde_json::from_str(&data[..]).unwrap())
                },
                Err(_) => {
                    return Err("Error while reading the configuration file, configuration file won't be used...")
                }
            }
        }
    }

    pub fn save_config(config_file_path: &PathBuf, config: &TrelloBSTAPIConfig) -> Result<(), &'static str> {
        match File::create(config_file_path.as_path()) {
            Ok(file)  => {
                //TODO: better error reporting
                let     config_json = serde_json::to_string(&config).unwrap();
                let mut config_file = file;
                match config_file.write(&config_json.into_bytes()[..]) {
                    Ok(_)  => Ok(()),
                    Err(_) => Err("Error while saving the configuration file...")
                }
            }
            Err(_)    => {
                Err("Cannot open configuration for saving...")
            }
        }
    }
}


