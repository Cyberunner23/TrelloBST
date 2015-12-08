
use std::io;

use config;

pub fn setup(config: &mut config::TrelloBSTAPIConfig) {

    if config.trello_api_key.is_empty() || config.trello_app_token.is_empty() {
        println!("Setting up Trello API Keys...");
    }

    if config.trello_api_key.is_empty() {
        println!("Log into Trello and enter your API key from https://trello.com/app-key : ");
        match io::stdin().read_line(&mut config.trello_api_key) {
            Ok(_)  => {config.trello_api_key = config.trello_api_key.trim_matches('\n').to_string();},
            Err(_) => {panic!("Error while reading the input.");}
        }
    }

    if config.trello_app_token.is_empty(){
        println!("Enter your app token from https://trello.com/1/authorize?response_type=token&key={}&scope=read%2Cwrite&expiration=never&name=TrelloBST : ", config.trello_api_key);
        match io::stdin().read_line(&mut config.trello_app_token) {
            Ok(_)  => {config.trello_app_token = config.trello_app_token.trim_matches('\n').to_string();},
            Err(_) => {panic!("Error while reading the input.");}
        }
    }
}