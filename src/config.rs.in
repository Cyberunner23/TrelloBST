
use std::env;
use std::fs::{File, Metadata, OpenOptions};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};

extern crate rustc_serialize;
use rustc_serialize::json;


pub struct TrelloBSTConfigPath {
    pub config_path: PathBuf
}

impl TrelloBSTConfigPath {

    pub fn new() -> TrelloBSTConfigPath {
        TrelloBSTConfigPath {
            config_path: PathBuf::new(),
        }
    }

    pub fn try_default_config_path() -> Result<TrelloBSTConfigPath, &'static str> {

        //Check for an existing config file

        let mut trellobst_config_path = TrelloBSTConfigPath::new();
        let mut _home_dir_found:        bool;

        match env::home_dir() {
            Some(home_dir) => {
                trellobst_config_path.config_path = home_dir;
                _home_dir_found                   = true;
            },
            None           => {
                _home_dir_found = false;
            }
        };

        if _home_dir_found {
            trellobst_config_path.config_path.push(".TrelloBST.cfg");
            //Try to open the file with r/w permissions
            match OpenOptions::new().read(true).write(true).create(true).open(trellobst_config_path.config_path.as_path()) {
                Ok(_)  => {
                    Ok(trellobst_config_path)
                }
                Err(_) => {
                    Err("Cannot open/create configuration file at default location.")
                }
            }
        } else {
            Err("Cannot open/create configuration file at default location.")
        }
    }

    pub fn try_custom_config_path(config_path: &Path) -> Result<TrelloBSTConfigPath, &'static str>{
        //Try to open the file with r/w permissions
        match OpenOptions::new().read(true).write(true).create(true).open(config_path) {
            Ok(_)  => {
                Ok(TrelloBSTConfigPath{config_path: PathBuf::from(config_path)})
            }
            Err(_) => {
                Err("Cannot open/create configuration file at custom location.")
            }
        }
    }

}


#[derive(RustcDecodable, RustcEncodable)]
pub struct TrelloBSTAPIConfig {
    pub trello_api_key:      String,
    pub trello_app_token:    String,
    pub travis_access_token: String,
    pub appveyor_api_token:  String,
}

impl TrelloBSTAPIConfig {

    pub fn new() -> TrelloBSTAPIConfig {
        TrelloBSTAPIConfig {
            trello_api_key:      "".to_string(),
            trello_app_token:    "".to_string(),
            travis_access_token: "".to_string(),
            appveyor_api_token:  "".to_string(),
        }
    }

    pub fn from_file(file: &mut File) -> Result<TrelloBSTAPIConfig, &'static str>{
        let metadata: Metadata;
        match file.metadata() {
            Ok(_metadata)  => {
                metadata = _metadata;
            }
            Err(_) => {
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
                Ok(_) => {
                    //TODO: better error checking
                    return Ok(json::decode(&data[..]).unwrap())
                },
                Err(_) => {
                    return Err("Error while reading the configuration file, configuration file won't be used...")
                }
            }
        }
    }

    pub fn save_config(config_path: &TrelloBSTConfigPath, config: &TrelloBSTAPIConfig) -> Result<(), &'static str> {
        match File::create(config_path.config_path.as_path()) {
            Ok(file)  => {
                //TODO: better error reporting
                let     config_json = json::encode(&config).unwrap();
                let mut config_file = file;
                match config_file.write(&config_json.into_bytes()[..]) {
                    Ok(_)  => Ok(()),
                    Err(_) => Err("Error while saving the configuration file...")
                }
            }
            Err(_) => {
                Err("Cannot open configuration for saving...")
            }
        }
    }
}



