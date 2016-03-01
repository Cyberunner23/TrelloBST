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
use hyper::Client;
use hyper::client::IntoUrl;
use hyper::client::response::Response;
use hyper::header::Headers;
use hyper::Url;

use serde_json::Value;

extern crate openssl;
use self::openssl::crypto::pkey::{EncryptionPadding, PKey};
use self::openssl::ssl::error::SslError;

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
    pub trello_app_token: String,
    pub list_id:          String,
    pub build_pass_id:    String,
    pub build_fail_id:    String
}

#[allow(dead_code)]
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

    pub fn from_json(json_data: &String) -> Result<ParsedHooksResponse, &'static str> {

        let mut tmp_parsed_hooks_response = ParsedHooksResponse::new();

        //Parse
        let data: Value;
        match serde_json::from_str(&json_data){
            Ok(_data) => data = _data,
            Err(err)  => {
                return Err("Error parsing the JSON data")
            }
        }

        //Get JSON object
        let mut object: BTreeMap<String, Value>;
        match data.as_object().ok_or("Error: JSON data does not describe an object.") {
            Ok(_object) => {
                object  = _object.clone();
            },
            Err(err)    => {
                return Err(err);
            }
        }

        //Get "hooks" field
        let hooks_value: Value;
        match object.get("hooks").ok_or("Error: The \"hooks\" field has not been found in the JSON data.") {
            Ok(_hooks_value) => {
                hooks_value  = _hooks_value.clone();
            }
            Err(err)        => {
                return Err(err)
            }
        }

        //Get "hooks" content
        let hooks_array: Vec<Value>;
        match hooks_value.as_array().ok_or("Error: The \"hooks\" field does not describe an array.") {
            Ok(_hooks_array) => {
                hooks_array  = _hooks_array.clone();
            }
            Err(err)        => {
                return Err(err)
            }
        }

        //For each hook
        for hook in &hooks_array {

            //Get hook object
            let mut hook_object: BTreeMap<String, Value>;
            hook_object = BTreeMap::new();
            match hook.as_object().ok_or("Error: An entry in the \"hook\" field does not describe an object.") {
                Ok(_hook_object) => {
                    hook_object  = _hook_object.clone();
                },
                Err(err)    => {
                    return Err(err);
                }
            }

            //Get "id" value
            let id_value: Value;
            match hook_object.get("id").ok_or("Error: Failed to acquire the \"id\" field of a \"hook\" field.") {
                Ok(_id_value) => {
                    id_value  = _id_value.clone();
                }
                Err(err)        => {
                    return Err(err)
                }
            }

            let id_u64: u64;
            match id_value.as_u64().ok_or("Error: Failed to convert the value of the \"id\" field to a u64.") {
                Ok(_id_u64) => {
                    id_u64  = _id_u64;
                }
                Err(err)        => {
                    return Err(err)
                }
            }

            //Get "name" field
            let name_value: Value;
            match hook_object.get("name").ok_or("Error: Failed to acquire the \"name\" field of a \"hook\" field.") {
                Ok(_name_value) => {
                    name_value  = _name_value.clone();
                }
                Err(err)        => {
                    return Err(err)
                }
            }

            let mut name_string = String::new();
            match name_value.as_string().ok_or("Error: Failed to convert the value of the \"name\" field to a string.") {
                Ok(_name_string) => {
                    name_string.push_str(_name_string.clone());
                }
                Err(err)        => {
                    return Err(err)
                }
            }

            //Get "owner_name" field
            let owner_name_value: Value;
            match hook_object.get("owner_name").ok_or("Error: Failed to acquire the \"owner_name\" field of a \"hook\" field.") {
                Ok(_owner_name_value) => {
                    owner_name_value  = _owner_name_value.clone();
                }
                Err(err)        => {
                    return Err(err)
                }
            }

            let mut owner_name_string = String::new();
            match owner_name_value.as_string().ok_or("Error: Failed to convert the value of the \"owner_name\" field to a string.") {
                Ok(_owner_name_string) => {
                    owner_name_string.push_str(_owner_name_string.clone());
                }
                Err(err)        => {
                    return Err(err)
                }
            }

            //Get "active" field and if null, assume false.
            let active_value: Value;
            match hook_object.get("active").ok_or("Error: Failed to acquire the \"active\" field of a \"hook\" field.") {
                Ok(_active_value) => {
                    active_value  = _active_value.clone();
                }
                Err(err)        => {
                    return Err(err)
                }
            }

            let active_bool: bool;
            if active_value.is_null() {
                active_bool = false;
            } else {
                match active_value.as_boolean().ok_or("Error: Failed to convert the value of the \"active\" field to a bool.") {
                    Ok(_active_bool) => {
                        active_bool  = _active_bool;
                    }
                    Err(err)        => {
                        return Err(err)
                    }
                }
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
//                       Functions                        //
////////////////////////////////////////////////////////////

pub fn setup_api(term: &mut Box<term::StdoutTerminal>, config_file_path: &mut PathBuf, config: &mut config::TrelloBSTAPIConfig) -> Result<(), &'static str> {

    if config.travis_access_token.is_empty() {

        //Get github token
        let mut github_token = String::new();
        if *config_file_path != PathBuf::new() {
            get_input_string!(term, &mut github_token, "Travis-CI currently uses a GitHub access token to authenticate and generate an API access token,
    please log into GitHub and go to https://github.com/settings/tokens and generate a new
    token and input it here. Note that \"read:org\", \"user:email\", \"repo_deployment\",
    \"repo:status\", and \"write:repo_hook\" scopes are required. Also, once the Travis-CI API key is acquired, the GitHub
    access token can be deleted.): ");
        } else {
            get_input_string!(term, &mut github_token, "Travis-CI currently uses a GitHub access token to authenticate and generate an API access token,
    please log into GitHub and go to https://github.com/settings/tokens and generate a new
    token and input it here. Note that \"read:org\", \"user:email\", \"repo_deployment\",
    \"repo:status\", and \"write:repo_hook\" scopes are required: ");
        }

        //NOTE: Removes spaces from the github token, (copying a token from the github page adds a space in front...)
        github_token = github_token.trim_matches(' ').to_string();

        //Convert github token to travis api key
        let mut api_call                = format!("https://api.travis-ci.org/auth/github?github_token={}", github_token);
        let mut response_body           = String::new();
        let mut header                  = Headers::new();
        let mut content_length: Vec<u8> = Vec::new();

        content_length.push(20 + github_token.len() as u8);
        header.set_raw("User-Agent",     vec![b"Travis_TrelloBST/1.0.0".to_vec()]);
        header.set_raw("Accept",         vec![b"application/vnd.travis-ci.2+json".to_vec()]);
        header.set_raw("Host",           vec![b"api.travis-ci.org".to_vec()]);
        header.set_raw("Content-Type",   vec![b"application/json".to_vec()]);
        header.set_raw("Content-Length", vec![content_length]);

        match utils::rest_api_call_post_with_header(&api_call, header) {
            Ok(_response_body) => response_body = _response_body,
            Err(err)           => {
                *config_file_path = PathBuf::new();
                return Err(err)
            }
        }

        match utils::get_single_json_value_as_string(&response_body, "access_token") {
            Ok(value) => config.travis_access_token = value,
            Err(err)  => return Err(err)
        }
    }
    Ok(())
}

pub fn create_travis_yml(term: &mut Box<term::StdoutTerminal>, config: &config::TrelloBSTAPIConfig, board_info: &mut trello::TrelloBoardInfo, ci_config_output_dir: &PathBuf) -> Result<(), &'static str> {

    //Get repo tag and public key
    let mut crypto_state = PKey::new();
    let mut repo_tag     = String::new();
    get_repo_tag_and_pub_key(term, config, &mut crypto_state, &mut repo_tag);

    //Encrypt Variables
    let status         = utils::StatusPrint::from_str(term, "Encrypting Trello API values.");
    let encrypted_vars = encrypt_vars(board_info, &config, &mut crypto_state);
    status.success(term);

    //Generate file
    generate_file(term, ci_config_output_dir, &encrypted_vars, &repo_tag)

}

pub fn get_repo_tag_and_pub_key(term: &mut Box<term::StdoutTerminal>, config: &config::TrelloBSTAPIConfig, crypto_state: &mut PKey, repo_tag: &mut String) {

    //Get repos.
    let      status = utils::StatusPrint::from_str(term, "Acquiring the repo list from Travis-CI.");
    let mut  hooks  = ParsedHooksResponse::new();
    match acquire_hooks(term, config)  {
        Ok(_hooks) => {
            hooks = _hooks;
            status.success(term);
        }
        Err(err) => {
            status.error(term);
            println!("{}", err);
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

            if option <= counter && option >= 0 {
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
            let mut auth          = format!("token {}", config.travis_access_token);
            let mut response_body = String::new();
            let mut header        = Headers::new();

            header.set_raw("User-Agent",    vec![b"Travis_TrelloBST/1.0.0".to_vec()]);
            header.set_raw("Accept",        vec![b"application/vnd.travis-ci.2+json".to_vec()]);
            header.set_raw("Authorization", vec![auth.into_bytes()]);
            header.set_raw("Host",          vec![b"api.travis-ci.org".to_vec()]);
            header.set_raw("Content-Type",  vec![b"application/json".to_vec()]);

            let mut is_api_call_success = true;
            match utils::rest_api_call_put_with_header(&api_call, header) {
                Ok(_response_body) => {
                    status.success(term);
                    response_body = _response_body
                }
                Err(err)           => {
                    status.error(term);
                    writeln_red!(term, "There was an error linking the {} to travis-CI: {}", hook.name, err);
                }
            }
        }

        *repo_tag = format!("{}/{}", hook.owner_name, hook.name);

        //Public Key
        let     status              = utils::StatusPrint::from_str(term, "Acquiring repo's public RSA key.");
        let     api_call            = format!("https://api.travis-ci.org/repos/{}/key", repo_tag);
        let mut response_body       = String::new();
        let mut is_api_call_success = true;

        match utils::rest_api_call_get(&api_call) {
            Ok(_response_body) => {
                status.success(term);
                response_body = _response_body
            }
            Err(err)           => {
                status.error(term);
                is_api_call_success = false;
                writeln_red!(term, "There was an error getting the public encryption key for {}: {}", repo_tag, err);
            }
        }

        let mut repo_response = RepoResponse::new();
        match serde_json::from_str(&response_body) {
            Ok(_repo_response) => repo_response = _repo_response,
            Err(_)             => {
                is_api_call_success = false;
                writeln_red!(term, "There was an error parsing the api response.");
            }
        }

        repo_response.key = repo_response.key.replace("-----BEGIN RSA PUBLIC KEY-----", "-----BEGIN PUBLIC KEY-----");
        repo_response.key = repo_response.key.replace("-----END RSA PUBLIC KEY-----", "-----END PUBLIC KEY-----");

        if is_api_call_success {
            let mut buff = Cursor::new(repo_response.key.as_bytes());
            match PKey::public_key_from_pem(&mut buff) {
                Ok(_crypto_state) => {
                    *crypto_state = _crypto_state;
                    break;
                }
                Err(err)          => {
                    match err {
                        SslError::OpenSslErrors(err) => {
                            for errs in err.iter() {
                                writeln_red!(term, "There was an error parsing the encryption key: {:?}", errs);
                            }
                        }
                        _ => ()
                    }
                }
            }
        }
    }
}

pub fn encrypt_vars(board_info: &mut trello::TrelloBoardInfo, config: &config::TrelloBSTAPIConfig, crypto_state: &mut PKey) -> TravisEncryptedVars{

    //Create environment variables
    let trello_app_token_env_var = format!("TRELLO_API_TOKEN={}",         config.trello_app_token);
    let list_id_env_var          = format!("TRELLO_API_LIST_ID={}",       board_info.list_id);
    let build_pass_id_env_var    = format!("TRELLO_API_BUILD_PASS_ID={}", board_info.build_pass_id);
    let build_fail_id_env_var    = format!("TRELLO_API_BUILD_FAIL_ID={}", board_info.build_fail_id);

    //Encrypt environment variables
    TravisEncryptedVars {
        trello_app_token: crypto_state.public_encrypt_with_padding(&trello_app_token_env_var.into_bytes(), EncryptionPadding::PKCS1v15).to_base64(base64::STANDARD),
        list_id:          crypto_state.public_encrypt_with_padding(&list_id_env_var.into_bytes(),          EncryptionPadding::PKCS1v15).to_base64(base64::STANDARD),
        build_pass_id:    crypto_state.public_encrypt_with_padding(&build_pass_id_env_var.into_bytes(),    EncryptionPadding::PKCS1v15).to_base64(base64::STANDARD),
        build_fail_id:    crypto_state.public_encrypt_with_padding(&build_fail_id_env_var.into_bytes(),    EncryptionPadding::PKCS1v15).to_base64(base64::STANDARD),
    }
}

#[allow(unused_assignments)]
pub fn generate_file(term: &mut Box<term::StdoutTerminal>, ci_config_output_dir: &PathBuf, encrypted_vars: &TravisEncryptedVars, repo_tag: &String) -> Result<(), &'static str> {

    let status = utils::StatusPrint::from_str(term, "Generating .travis.yml");
    let mut travis_file: File;
    let mut local_ci_config_output_dir = ci_config_output_dir.clone();
    local_ci_config_output_dir.push(".travis.yml");
    match File::create(local_ci_config_output_dir.as_path()) {
        Ok(_travis_file)  => {
            travis_file = _travis_file;
        }
        Err(_)    => {
            status.error(term);
            return Err("Failed to create .travis.yml");
        }
    }

    let mut file_data = String::new();
    file_data = format!(
    "language:
sudo: false
os:
  - linux

install:
script:

env:
  global:
    - BUILD_DIRECTORY=./
    - secure: \"{1}\"
    - secure: \"{2}\"
    - secure: \"{3}\"
    - secure: \"{4}\"

after_success:
  - if [ ${{TRAVIS_SECURE_ENV_VARS}} = true ] ; then
         tar -zcf build.tar.gz ${{BUILD_DIRECTORY}}
      && buildLink=$(curl --upload-file ./build.tar.gz https://transfer.sh/build.tar.gz)
      && travis_branch=\"[\"${{TRAVIS_BRANCH}}\"]\"
      && ci_name=\"[Travis-CI]\"
      && os_name=\"[\"${{TRAVIS_OS_NAME}}\"]\"
      && compiler=\"[\"${{CXXCOMPILER}}\"]:\"
      && pass=\"%20#\"${{TRAVIS_BUILD_NUMBER}}\"%20PASSED\"
      && message=${{travis_branch}}${{ci_name}}${{os_name}}${{compiler}}${{pass}}
      && card_name=\"name=\"${{message}}
      && additional_data=\"&due=null&pos=top\"
      && description=\"&desc=\\[Build\\]:%20\"${{buildLink}}\"%0D\\[Logs\\]:%20https://travis-ci.org/{5}/jobs/\"${{TRAVIS_JOB_ID}}
      && trello_data=\"&idList=\"${{TRELLO_API_LIST_ID}}\"&idLabels=\"${{TRELLO_API_BUILD_PASS_ID}}\"&token=\"${{TRELLO_API_TOKEN}}\"&key={0}
      && data=${{card_name}}${{additional_data}}${{description}}${{trello_data}}
      && curl -s -o /dev/null -w \"%{{http_code}}\\n\" --data ${{data}} https://api.trello.com/1/cards;
    fi

after_failure:
  - if [ ${{TRAVIS_SECURE_ENV_VARS}} = true ] ; then
         travis_branch=\"[\"${{TRAVIS_BRANCH}}\"]\"
      && ci_name=\"[Travis-CI]\"
      && os_name=\"[\"${{TRAVIS_OS_NAME}}\"]\"
      && compiler=\"[\"${{CXXCOMPILER}}\"]:\"
      && fail=\"%20#\"${{TRAVIS_BUILD_NUMBER}}\"%20FAILED\"
      && message=${{travis_branch}}${{ci_name}}${{os_name}}${{compiler}}${{fail}}
      && card_name=\"name=\"${{message}}
      && additional_data=\"&due=null&pos=top\"
      && description=\"&desc=\\[Logs\\]:%20https://travis-ci.org/{5}/jobs/\"${{TRAVIS_JOB_ID}}\"
      && trello_data=\"&idList=\"${{TRELLO_API_LIST_ID}}\"&idLabels=\"${{TRELLO_API_BUILD_FAIL_ID}}\"&token=\"${{TRELLO_API_TOKEN}}\"&key={0}
      && data=${{card_name}}${{additional_data}}${{description}}${{trello_data}}
      && curl -s -o /dev/null -w \"%{{http_code}}\\n\" --data ${{data}} https://api.trello.com/1/cards;
    fi
", config::trello_api_key,
   encrypted_vars.trello_app_token,
   encrypted_vars.list_id,
   encrypted_vars.build_pass_id,
   encrypted_vars.build_fail_id,
   repo_tag);

    match travis_file.write_all(&file_data.into_bytes()[..]) {
        Ok(()) => (),
        Err(_) => {
            status.error(term);
            return Err("Error while writing to the file.");
        }
    }

    match travis_file.flush() {
        Ok(()) => (),
        Err(_) => {
            status.error(term);
            return Err("Error while flushing the file writing buffer.")
        }
    }

    status.success(term);
    Ok(())
}

pub fn acquire_hooks(term: &mut Box<term::StdoutTerminal>, config: &config::TrelloBSTAPIConfig) -> Result<ParsedHooksResponse, &'static str> {

    let mut api_call      = format!("https://api.travis-ci.org/hooks");
    let mut auth          = format!("token {}", config.travis_access_token);
    let mut response_body = String::new();
    let mut header        = Headers::new();

    header.set_raw("User-Agent",    vec![b"Travis_TrelloBST/1.0.0".to_vec()]);
    header.set_raw("Accept",        vec![b"application/vnd.travis-ci.2+json".to_vec()]);
    header.set_raw("Authorization", vec![auth.into_bytes()]);
    header.set_raw("Host",          vec![b"api.travis-ci.org".to_vec()]);
    header.set_raw("Content-Type",  vec![b"application/json".to_vec()]);

    match utils::rest_api_call_get_with_header(&api_call, header) {
        Ok(_response_body) => response_body = _response_body,
        Err(err)           => return Err(err)
    }

    match ParsedHooksResponse::from_json(&response_body) {
        Ok(hooks) => Ok(hooks),
        Err(err)  => return Err(err)
    }
}





















