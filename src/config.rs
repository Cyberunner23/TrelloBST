
use std::env;
use std::fs::{File, OpenOptions};
use std::path::{Path, PathBuf};

extern crate rustc_serialize;
use rustc_serialize::json;

pub struct TrelloBSTConfigPath {
    pub config_path: PathBuf
}

#[derive(RustcDecodable, RustcEncodable)]
pub struct TrelloBSTAPIConfig {
    pub trello_api_key:       String,
    pub trello_app_key:       String,
    pub travis_access_token: String,
    pub appveyor_api_token:  String,
}


impl TrelloBSTConfigPath {

    pub fn try_default_config_path() -> Result<TrelloBSTConfigPath, &'static str> {

        //Check for an existing config file

        let mut _home_dir:       PathBuf;
        let mut _home_dir_found: bool;
        _home_dir = PathBuf::new();

        match env::home_dir() {
            Some(home_dir) => {
                _home_dir       = home_dir;
                _home_dir_found = true
            },
            None           => {
                _home_dir_found = false;
            }
        };

        if _home_dir_found {
            _home_dir.push(".TrelloBST.cfg");
            //Try to open the file with r/w permissions
            match OpenOptions::new().read(true).write(true).create(true).open(_home_dir.as_path()) {
                Ok(_) => {
                    Ok(TrelloBSTConfigPath{config_path: PathBuf::from(_home_dir)})
                }
                Err(_)           => {
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






