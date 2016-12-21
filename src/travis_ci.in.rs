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
use std::io;
use std::io::{Cursor, Write};
use std::fs::File;
use std::path::PathBuf;
use std::process::exit;

extern crate rustc_serialize;
use self::rustc_serialize::base64::{self, ToBase64};

extern crate term;


extern crate hyper;
use hyper::header::Headers;

use serde_json::Value;

extern crate openssl;
use self::openssl::crypto::pkey::{EncryptionPadding, PKey};
use self::openssl::ssl::error::SslError;

use ci::CITrait;
use config;
use trello;
use utils;


////////////////////////////////////////////////////////////
//                         Macros                         //
////////////////////////////////////////////////////////////

include!("utils_macros.rs");


////////////////////////////////////////////////////////////
//                        Structs                         //
////////////////////////////////////////////////////////////

pub struct TravisEncryptedVars {
    pub trello_api_token: String,
    pub list_id:          String,
    pub build_pass_id:    String,
    pub build_fail_id:    String
}

#[derive(Deserialize)]
pub struct RepoResponse {
    key:  String,
    fingerprint: String
}


pub struct ParsedHooksResponse {
    hooks: Vec<Hook>
}

#[derive(Clone)]
pub struct Hook {
    id:         u64,
    name:       String,
    owner_name: String,
    active:     bool
}


////////////////////////////////////////////////////////////
//                         Impls                          //
////////////////////////////////////////////////////////////

impl RepoResponse {
    pub fn new() -> RepoResponse {
        RepoResponse {
            key:         String::new(),
            fingerprint: String::new()
        }
    }
}

impl ParsedHooksResponse {

    pub fn new() -> ParsedHooksResponse {
        ParsedHooksResponse{
            hooks: Vec::new()
        }
    }

    pub fn from_json(json_data: &String) -> Result<ParsedHooksResponse, String> {

        let mut tmp_parsed_hooks_response = ParsedHooksResponse::new();

        //Parse
        let data: Value = match serde_json::from_str(&json_data){
            Ok(data) => data,
            Err(_)   => {return Err("Error parsing the JSON data".to_string());}
        };

        //Get JSON object
        let object = try!(data.as_object().ok_or("Error: JSON data does not describe an object.".to_string()));

        //Get "hooks" field
        let hooks_value = try!(object.get("hooks").ok_or("Error: The \"hooks\" field has not been found in the JSON data.".to_string()));

        //Get "hooks" content
        let hooks_array: Vec<Value> = match hooks_value.as_array().ok_or("Error: The \"hooks\" field does not describe an array.".to_string()) {
            Ok(hooks_array) => hooks_array.clone(),
            Err(err)        => {return Err(err);}
        };

        //For each hook
        for hook in &hooks_array {

            //Get hook object
            let hook_object = try!(hook.as_object().ok_or("Error: An entry in the \"hook\" field does not describe an object.".to_string())).clone();

            //Get "id" value
            let id_value = try!(hook_object.get("id").ok_or("Error: Failed to acquire the \"id\" field of a \"hook\" field.".to_string())).clone();
            let id_u64   = try!(id_value.as_u64().ok_or("Error: Failed to convert the value of the \"id\" field to a u64.".to_string()));

            //Get "name" field
            let name_value  = try!(hook_object.get("name").ok_or("Error: Failed to acquire the \"name\" field of a \"hook\" field.".to_string())).clone();
            let name_string = try!(name_value.as_str().ok_or("Error: Failed to convert the value of the \"name\" field to a string.".to_string())).to_string();

            //Get "owner_name" field
            let owner_name_value  = try!(hook_object.get("owner_name").ok_or("Error: Failed to acquire the \"owner_name\" field of a \"hook\" field.".to_string())).clone();
            let owner_name_string = try!(owner_name_value.as_str().ok_or("Error: Failed to convert the value of the \"owner_name\" field to a string.".to_string())).to_string();

            //Get "active" field and if null, assume false.
            let active_value = try!(hook_object.get("active").ok_or("Error: Failed to acquire the \"active\" field of a \"hook\" field.".to_string())).clone();

            let active_bool: bool;
            if active_value.is_null() {
                active_bool = false;
            } else {
                active_bool = try!(active_value.as_bool().ok_or("Error: Failed to convert the value of the \"active\" field to a bool.".to_string()));
            }

            tmp_parsed_hooks_response.hooks.push(Hook{
                id:         id_u64,
                name:       name_string,
                owner_name: owner_name_string,
                active:     active_bool
            });
        }
        Ok(tmp_parsed_hooks_response)
    }
}


////////////////////////////////////////////////////////////
//                         Travis                         //
////////////////////////////////////////////////////////////

#[derive(Clone)]
pub struct TravisCI{}

impl CITrait for TravisCI{
    fn get_filename(&mut self) -> String {return ".travis.yml".to_string();}

    fn get_name(&mut self) -> String {return "Travis-CI".to_string();}

    fn setup(&mut self, term: &mut Box<term::StdoutTerminal>, config: &mut config::TrelloBSTConfig) -> Result<(), String> {

        let travis_access_token = config.get("travis_access_token");

        if travis_access_token.is_empty() {

            //Get github token
            let mut github_token = String::new();
            if travis_access_token.is_empty() {
                get_input_string!(term, &mut github_token, "Travis-CI currently uses a GitHub access token to authenticate and generate an API access token,
    please log into GitHub and go to https://github.com/settings/tokens and generate a new
    token and input it here. Note that \"read:org\", \"user:email\", \"repo_deployment\",
    \"repo:status\", and \"write:repo_hook\" scopes are required: ");
            }

            //NOTE: Removes spaces from the github token, (copying a token from the github page adds a space in front...)
            github_token = github_token.trim_matches(' ').to_string();

            //Convert github token to travis api key
            let     api_call                = format!("https://api.travis-ci.org/auth/github?github_token={}", github_token);
            let mut header                  = Headers::new();
            let mut content_length: Vec<u8> = Vec::new();

            content_length.push(20 + github_token.len() as u8);
            header.set_raw("User-Agent",     vec![b"Travis_TrelloBST/1.0.0".to_vec()]);
            header.set_raw("Accept",         vec![b"application/vnd.travis-ci.2+json".to_vec()]);
            header.set_raw("Host",           vec![b"api.travis-ci.org".to_vec()]);
            header.set_raw("Content-Type",   vec![b"application/json".to_vec()]);
            header.set_raw("Content-Length", vec![content_length]);

            let response_body = match utils::rest_api_call_post_with_header(&api_call, header) {
                Ok(response_body) => response_body,
                Err(err)          => {return Err(err.to_string())}
            };

            config.set("travis_access_token", &try!(utils::get_single_json_value_as_string(&response_body, "access_token"))[..]);
        }
        Ok(())
    }

    fn generate_ci_config(&mut self, term: &mut Box<term::StdoutTerminal>, config: &mut config::TrelloBSTConfig) -> Result<(String, String), String> {

        //Get repo tag and public key
        let mut crypto_state = PKey::new();
        match self.get_repo_pub_key(term, config, &mut crypto_state) {
            Ok(())   => (),
            Err(err) => {return Err(err);}
        }

        //Encrypt Variables
        let status         = utils::StatusPrint::from_str(term, "Encrypting Trello API values.");
        let encrypted_vars = self.encrypt_vars(config, &mut crypto_state);
        status.success(term);

        //Generate
        let mut file_data = include_str!("templates/travis-ci").to_string();

        file_data = file_data.replace("<TRELLO_API_TOKEN>",         &encrypted_vars.trello_api_token[..]);
        file_data = file_data.replace("<TRELLO_API_LIST_ID>",       &encrypted_vars.list_id[..]);
        file_data = file_data.replace("<TRELLO_API_BUILD_PASS_ID>", &encrypted_vars.build_pass_id[..]);
        file_data = file_data.replace("<TRELLO_API_BUILD_FAIL_ID>", &encrypted_vars.build_fail_id[..]);

        return Ok((self.get_filename(), file_data));
    }
}

impl TravisCI{

    pub fn get_repo_pub_key(&mut self, term: &mut Box<term::StdoutTerminal>, config: &mut config::TrelloBSTConfig, crypto_state: &mut PKey) -> Result<(), String>{

        //Get repos.
        let     status   = utils::StatusPrint::from_str(term, "Acquiring the repo list from Travis-CI.");
        let mut repo_tag = String::new();
        let mut hooks    = ParsedHooksResponse::new();
        match self.acquire_hooks(config)  {
            Ok(_hooks) => {
                hooks = _hooks;
                status.success(term);
            }
            Err(err) => {
                status.error(term);
                return Err(err);
            }
        }

        loop{

            //Select repo.
            println!("Which repo do you want the .travis.yml file for?");

            let mut counter = 1;
            for i in 0..hooks.hooks.len() {
                println!("[{}] {}", i + 1, hooks.hooks[i].name);
                counter += 1;
            }
            writeln_red!(term, "[0] Quit.",);

            let mut option: usize = 0;
            loop {

                get_input_usize!(term, &mut option, "Please enter an option: ");

                if option <= counter {
                    break;
                }else {
                    writeln_red!(term, "Please enter a valid option.");
                }
            }

            let hook: Hook;
            if option == 0 {
                exit(0);
            } else {
                hook = hooks.hooks[option - 1].clone();
            }

            //Link repo.
            if !hook.active {
                let     status        = utils::StatusPrint::from_str(term, "Linking repo to Travis-CI.");
                let     api_call      = format!("https://api.travis-ci.org/hooks/{}?hook[active]=true", hook.id);
                let     auth          = format!("token {}", config.get("travis_access_token"));
                let mut response_body = String::new();
                let mut header        = Headers::new();

                header.set_raw("User-Agent",    vec![b"Travis_TrelloBST/1.0.0".to_vec()]);
                header.set_raw("Accept",        vec![b"application/vnd.travis-ci.2+json".to_vec()]);
                header.set_raw("Authorization", vec![auth.into_bytes()]);
                header.set_raw("Host",          vec![b"api.travis-ci.org".to_vec()]);
                header.set_raw("Content-Type",  vec![b"application/json".to_vec()]);

                match utils::rest_api_call_put_with_header(&api_call, header) {
                    Ok(_response_body) => {
                        status.success(term);
                        response_body = _response_body
                    }
                    Err(err)           => {
                        status.error(term);
                        return Err(format!("There was an error linking the {} to travis-CI: {}", hook.name, err));
                    }
                }
            }

            repo_tag = format!("{}/{}", hook.owner_name, hook.name);

            //Public Key
            let     status              = utils::StatusPrint::from_str(term, "Acquiring repo's public RSA key.");
            let     api_call            = format!("https://api.travis-ci.org/repos/{}/key", repo_tag);
            let mut is_api_call_success = true;
            let mut response_body       = String::new();

            match utils::rest_api_call_get(&api_call) {
                Ok(_response_body) => {
                    status.success(term);
                    response_body = _response_body
                }
                Err(err)           => {
                    status.error(term);
                    is_api_call_success = false;
                    return Err(format!("There was an error getting the public encryption key for {}: {}", repo_tag, err));
                }
            }

            let mut repo_response: RepoResponse = match serde_json::from_str(&response_body) {
                Ok(response) => response,
                Err(_)       => {
                    is_api_call_success = false;
                    return Err("There was an error parsing the api response.".to_string());
                }
            };

            repo_response.key = repo_response.key.replace("-----BEGIN RSA PUBLIC KEY-----", "-----BEGIN PUBLIC KEY-----");
            repo_response.key = repo_response.key.replace("-----END RSA PUBLIC KEY-----", "-----END PUBLIC KEY-----");

            if is_api_call_success {
                let mut buff = Cursor::new(repo_response.key.as_bytes());
                match PKey::public_key_from_pem(&mut buff) {
                    Ok(_crypto_state) => {
                        *crypto_state = _crypto_state;
                        return Ok(());
                    }
                    Err(err)          => {
                        match err {
                            SslError::OpenSslErrors(err) => {
                                for errs in err.iter() {
                                    return Err(format!("There was an error parsing the encryption key: {:?}", errs));
                                }
                            }
                            _ => ()
                        }
                    }
                }
            }

        }
    }

    pub fn encrypt_vars(&mut self, config: &mut config::TrelloBSTConfig, crypto_state: &mut PKey) -> TravisEncryptedVars{

        //Get config values
        let trello_api_token = config.get("trello_api_token");
        let list_id          = config.get("trello_list_id");
        let build_pass_id    = config.get("trello_label_pass_id");
        let build_fail_id    = config.get("trello_label_fail_id");

        //Create environment variables
        let trello_api_token_env_var = format!("TRELLO_API_TOKEN={}",         trello_api_token);
        let list_id_env_var          = format!("TRELLO_API_LIST_ID={}",       list_id);
        let build_pass_id_env_var    = format!("TRELLO_API_BUILD_PASS_ID={}", build_pass_id);
        let build_fail_id_env_var    = format!("TRELLO_API_BUILD_FAIL_ID={}", build_fail_id);

        //Encrypt environment variables
        TravisEncryptedVars {
            trello_api_token: crypto_state.public_encrypt_with_padding(&trello_api_token_env_var.into_bytes(), EncryptionPadding::PKCS1v15).to_base64(base64::STANDARD),
            list_id:          crypto_state.public_encrypt_with_padding(&list_id_env_var.into_bytes(),          EncryptionPadding::PKCS1v15).to_base64(base64::STANDARD),
            build_pass_id:    crypto_state.public_encrypt_with_padding(&build_pass_id_env_var.into_bytes(),    EncryptionPadding::PKCS1v15).to_base64(base64::STANDARD),
            build_fail_id:    crypto_state.public_encrypt_with_padding(&build_fail_id_env_var.into_bytes(),    EncryptionPadding::PKCS1v15).to_base64(base64::STANDARD)
        }
    }

    pub fn acquire_hooks(&mut self, config: &mut config::TrelloBSTConfig) -> Result<ParsedHooksResponse, String> {

        let     api_call      = format!("https://api.travis-ci.org/hooks");
        let     auth          = format!("token {}", config.get("travis_access_token"));
        let mut header        = Headers::new();

        header.set_raw("User-Agent",    vec![b"Travis_TrelloBST/1.0.0".to_vec()]);
        header.set_raw("Accept",        vec![b"application/vnd.travis-ci.2+json".to_vec()]);
        header.set_raw("Authorization", vec![auth.into_bytes()]);
        header.set_raw("Host",          vec![b"api.travis-ci.org".to_vec()]);
        header.set_raw("Content-Type",  vec![b"application/json".to_vec()]);

        let response_body = try!(utils::rest_api_call_get_with_header(&api_call, header));

        Ok(try!(ParsedHooksResponse::from_json(&response_body)))
    }
}
