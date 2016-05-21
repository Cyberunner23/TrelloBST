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


use std::collections::BTreeMap;
use std::error::Error;
use std::fs::{File, Metadata, OpenOptions};
use std::io::{Read, Write};
use std::path::PathBuf;

extern crate term;

use serde_json::Value;

mod utils;


////////////////////////////////////////////////////////////
//                         Macros                         //
////////////////////////////////////////////////////////////

include!("utils_macros.rs");


////////////////////////////////////////////////////////////
//                        Constants                       //
////////////////////////////////////////////////////////////
pub static TRELLO_API_KEY: &'static str = "0e190833c4db5fd7d3b0b26ae642d6fa";


////////////////////////////////////////////////////////////
//                        Structs                         //
////////////////////////////////////////////////////////////

#[derive(Clone)]
pub struct TrelloBSTConfig {
    pub key_val_map: BTreeMap<String, String>,
    pub config_mode: Option<PathBuf>
}

#[derive(Serialize, Deserialize)]
pub struct TrelloBSTAPIConfig {
    pub trello_app_token:    String,
    pub travis_access_token: String,
    pub appveyor_api_token:  String,
}


////////////////////////////////////////////////////////////
//                         Impls                          //
////////////////////////////////////////////////////////////

impl TrelloBSTConfig {

    pub fn load(&mut self, config_mode: Option<PathBuf>) -> Result<(), &'static str> {

        self.config_mode = config_mode;

        //Parse if we're using a config file, silently skip if were not
        if self.config_mode.is_some() {

            //Load file
            let mut file: File;
            match File::open(self.clone().config_mode.unwrap().as_path()) {
                Ok(_file) => {file = _file;}
                Err(_)    =>{
                    self.config_mode = Option::None;
                    return Err("Error: Failed to open the configuration file for parsing, TrelloBST will continue without saving inputted values into the configuration file.");
                }
            }

            //Get config file metadata.
            let metadata: Metadata;
            match file.metadata() {
                Ok(_metadata)  => {metadata = _metadata;}
                Err(_)         => {
                    self.config_mode = Option::None;
                    return Err("Error: Failed to gather metadata of the configuration file, TrelloBST will continue without saving inputted values into the configuration file.")
                }
            }

            //Parse config file
            let file_length: usize = metadata.len() as usize;
            if file_length == 0 {
                self.key_val_map = BTreeMap::new();
            } else {

                //Read file
                let mut file_data: String = String::with_capacity(file_length +1);
                match file.read_to_string(&mut file_data) {
                    Ok(_)  => (),
                    Err(_) => {
                        self.config_mode = Option::None;
                        return Err("Error: Failed to read the configuration file, TrelloBST will continue without saving inputted values into the configuration file.")
                    }
                }


                //Parse
                let json_data: Value;
                match serde_json::from_str(&file_data){
                    Ok(_json_data) => json_data = _json_data,
                    Err(_)  => {
                        self.config_mode = Option::None;
                        return Err("Error: Failed to parse the JSON data in the configuration file, TrelloBST will continue without saving inputted values into the configuration file.")
                    }
                }


                //Extract data
                //Get JSON object
                let json_object: BTreeMap<String, Value>;
                match json_data.as_object().ok_or("Error: JSON data in the configuration file does not describe a JSON object, TrelloBST will continue without saving inputted values into the configuration file.") {
                    Ok(_json_object) => {json_object  = _json_object.clone();},
                    Err(err)         => {
                        self.config_mode = Option::None;
                        return Err(err);
                    }
                }

                //Iterate through object
                for (key, val) in &json_object {
                    if val.is_string() {
                        self.key_val_map.insert(key.clone(), val.as_string().unwrap().to_string());
                    } else {
                        println!("Value of the \"{}\" field in the configuration file is not a string, this value will not be considered.", key);
                    }
                }
            }
        }
        Ok(())
    }


    //Save config
    pub fn save(&mut self) -> Result<(), String> {

        if self.config_mode.is_some() {

            let mut json_map:        BTreeMap<String, Value> = BTreeMap::new();
            let     json_map_string: String;

            for (key, val) in &self.key_val_map {
                json_map.insert(key.clone(), Value::String(val.clone()));
            }

            let value = Value::Object(json_map);

            match serde_json::to_string(&value) {
                Ok(_json_map_string) => {json_map_string = _json_map_string;}
                Err(err) => {
                    return Err(err.description().to_string());
                }
            }

            //Open file, overwrite config with what we have
            let mut file: File;
            match OpenOptions::new().write(true).truncate(true).open(self.config_mode.clone().unwrap().as_path()) {
                Ok(_file) => {
                    file = _file;
                    match file.write_all(json_map_string.as_bytes()) {
                        Ok(()) => (),
                        Err(_) => {
                            self.config_mode = Option::None;
                            return Err("Error: Failed to write data to the configuration file, TrelloBST will continue without saving inputted values into the configuration file.".to_string());
                        }
                    }
                }
                Err(_) => {
                    self.config_mode = Option::None;
                    return Err("Error: Failed to open the configuration file for saving, TrelloBST will continue without saving inputted values into the configuration file.".to_string());
                }
            }
        }
        Ok(())
    }


    //Sets a config key-value pair
    pub fn set(&mut self, key: &str, val: &str) {
        if self.config_mode.is_some() {
            self.key_val_map.insert(key.to_string(), val.to_string());
        }
    }


    //Gets a config value for a key, returns "" if key doesnt exist and creates the key, returns () if not using config file.
    pub fn get(&mut self, key: &str) -> Result<String, ()> {

        if self.config_mode.is_some() {
            if self.key_val_map.contains_key(&key.to_string()) {
                return Ok(self.key_val_map.get(&key.to_string()).unwrap().clone());
            } else {
                self.key_val_map.insert(key.to_string(), "".to_string());
                return Ok("".to_string());
            }
        } else {
            Err(())
        }
    }
}


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



