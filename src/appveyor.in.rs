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



use std::io::Read;


extern crate term;

extern crate hyper;
use hyper::Client;
use hyper::client::Body;
use hyper::client::IntoUrl;
use hyper::client::response::Response;
use hyper::header::Headers;
use hyper::Url;

use serde_json::Value;


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

#[derive(Deserialize)]
#[derive(Clone)]
pub struct Repositories {
    id:                  u64,
    #[serde(rename="groupName")]
    group_name:          String,
    #[serde(rename="groupType")]
    group_type:          String,
    #[serde(rename="groupAvatarUrl")]
    group_avatar_url:    String,
    name:                String,
    #[serde(rename="fullName")]
    full_name:           String,
    description:         String,
    #[serde(rename="isPrivate")]
    is_private:          bool,
    #[serde(rename="scmType")]
    scm_type:            String,
    #[serde(rename="masterBranch")]
    master_branch:       String,
    #[serde(rename="hasChildren")]
    has_children:        bool,
    #[serde(rename="showScmMoniker")]
    show_scm_moniker:    bool,
    #[serde(rename="showAccessMoniker")]
    show_sccess_moniker: bool,
}

#[derive(Deserialize)]
pub struct GroupInfo {
    name:         String,
    #[serde(rename="avatarUrl")]
    avatar_url:   String,
    #[serde(rename="groupType")]
    group_type:   String,
    repositories: Vec<Repositories>
}

#[derive(Deserialize)]
pub struct GithubResponse {
    users:         Vec<GroupInfo>,
    organizations: Vec<GroupInfo>
}

pub struct AppVeyorEncryptedVars {
    pub trello_app_token: String,
    pub list_id:          String,
    pub build_pass_id:    String,
    pub build_fail_id:    String
}

////////////////////////////////////////////////////////////
//                         Impls                          //
////////////////////////////////////////////////////////////

impl Repositories {
    pub fn new() -> Repositories {
        Repositories {
            id:                  0,
            group_name:          String::new(),
            group_type:          String::new(),
            group_avatar_url:    String::new(),
            name:                String::new(),
            full_name:           String::new(),
            description:         String::new(),
            is_private:          false,
            scm_type:            String::new(),
            master_branch:       String::new(),
            has_children:        false,
            show_scm_moniker:    false,
            show_sccess_moniker: false,
        }
    }
}

impl GithubResponse {

    pub fn new() -> GithubResponse {
        GithubResponse {
            users:         Vec::new(),
            organizations: Vec::new()
        }
    }

    pub fn from_api_call(&mut self, config: &config::TrelloBSTAPIConfig) -> Result<(), &'static str> {

        //Do API call.
        let     api_call      = format!("https://ci.appveyor.com/api/repositories/gitHub");
        let mut response_body = String::new();
        let mut header        = Headers::new();
        let     auth          = format!("Bearer {}", config.appveyor_api_token);

        header.set_raw("Authorization",  vec![auth.into_bytes()]);
        header.set_raw("Content-Type",   vec![b"application/json".to_vec()]);

        match utils::rest_api_call_get_with_header(&api_call, header) {
            Ok(_response_body) => response_body = _response_body,
            Err(err)           => return Err(err)
        }

        //Parse raw array.
        let data: Value;
        match serde_json::from_str(&response_body){
            Ok(_data) => data = _data,
            Err(err)  => {
                return Err("Error parsing the JSON data")
            }
        }

        //Get group Info.
        let group_info_array: Vec<Value>;
        match data.as_array() {
            Some(_group_info_array) => group_info_array = _group_info_array.clone(),
            None                    => return Err("Error: The JSON response from GithubResponse is not an array.")
        }

        for group in group_info_array {

            let group_info: BTreeMap<String, Value>;
            match group.as_object() {
                Some(_group_info) => group_info = _group_info.clone(),
                None              => return Err("Error: Expected an array of JSON objects in GithubResponse.")
            }

            let mut group_type_value: Value;
            match group_info.get("groupType") {
                Some(_group_type_value) => group_type_value = _group_type_value.clone(),
                None              => return Err("Error: Could not find the \"groupType\" field in a GithubResponse object.")
            }

            let mut group_type: String;
            match group_type_value.as_string() {
                Some(_group_type) => group_type = _group_type.clone().to_string(),
                None              => return Err("Error: Failed to parse the value of \"groupType\" in the GithubResponse object.")
            }

            if group_type == "user" {
                match serde_json::from_value(group) {
                    Ok(parsed_group) => self.users.push(parsed_group),
                    Err(_)           => return Err("Error while parsing a group info object in GithubResponse")
                }
            } else if group_type == "organization" {
                match serde_json::from_value(group) {
                    Ok(parsed_group) => self.organizations.push(parsed_group),
                    Err(_)           => return Err("Error while parsing a group info object in GithubResponse")
                }
            } else {
                return Err("Error: Invalid group type found in the \"groupType\" field in a GithubResponse object.")
            }
        }

        Ok(())
    }
}

////////////////////////////////////////////////////////////
//                       Functions                        //
////////////////////////////////////////////////////////////

pub fn setup_api(term: &mut Box<term::StdoutTerminal>, config_file_path: &mut PathBuf, config: &mut config::TrelloBSTAPIConfig){

    if config.appveyor_api_token.is_empty() {
        //Get appveyo api key
        get_input_string!(term, &mut config.appveyor_api_token, "TrelloBST currently supports repos on github only.
    Please log into appveyor and link your github account then go to https://ci.appveyor.com/api-token and enter
    your api token here: ");
    }
}


pub fn create_appveyor_yml(term: &mut Box<term::StdoutTerminal>, config: &config::TrelloBSTAPIConfig, board_info: &mut trello::TrelloBoardInfo, ci_config_output_dir: &PathBuf) -> Result<(), &'static str>{

    //Select Repo
    let mut repo_tag = String::new();
    match repo_selection(term, config, &mut repo_tag) {
        Ok(())   => (),
        Err(err) => return Err(err)
    }

    //Encrypt Variables
    let     status             = utils::StatusPrint::from_str(term, "Encrypting Trello API values.");
    let mut encrypted_variables: AppVeyorEncryptedVars;
    match encrypt_vars(&board_info, &config) {
        Ok(vars) => {
            status.success(term);
            encrypted_variables = vars;
        },
        Err(err) => {
            status.error(term);
            return Err(err)
        }
    }

    //Generate File
    let status = utils::StatusPrint::from_str(term, "Generating appveyor.yml");
    generate_file(term, ci_config_output_dir, &encrypted_variables)
}


pub fn repo_selection(term: &mut Box<term::StdoutTerminal>, config: &config::TrelloBSTAPIConfig, repo_tag: &mut String) -> Result<(), &'static str> {

    let mut status      = utils::StatusPrint::from_str(term, "Acquiring the repo list from AppVeyor.");
    let mut groups_info = GithubResponse::new();
    match groups_info.from_api_call(&config) {
        Ok(())   => status.success(term),
        Err(err) => {
            status.error(term);
            return Err(err)
        }
    }

    println!("Which repo do you want the appveyor.yml file for?");

    //Print options.
    let mut counter = 1;

    //Print user repos.
    let mut repos: BTreeMap<usize, Repositories> = BTreeMap::new();
    for user in 0..groups_info.users.len() {
        println!("User: {}", groups_info.users[user].name);
        for repo in 0..groups_info.users[user].repositories.len() {
            println!("[{}] {}", counter, groups_info.users[user].repositories[repo].name);
            repos.insert(counter, groups_info.users[user].repositories[repo].clone());
            counter += 1;
        }
    }

    //Print organization repos.
    for organization in 0..groups_info.organizations.len() {
        println!("Organization: {}", groups_info.organizations[organization].name);
        for repo in 0..groups_info.organizations[organization].repositories.len() {
            println!("[{}] {}", counter, groups_info.organizations[organization].repositories[repo].name);
            repos.insert(counter, groups_info.organizations[organization].repositories[repo].clone());
            counter += 1;
        }
    }
    writeln_red!(term, "[0] Quit.",);

    //Get input.
    let mut option: usize = 0;
    loop {
        get_input_usize!(term, &mut option, "Please enter an option: ");
        if option <= counter{
            break;
        }else {
            writeln_red!(term, "Please enter a valid option.");
        }
    }

    //Get selected repo.
    let mut repo =  Repositories::new();
    match repos.get(&option) {
        Some(_repo) => repo = _repo.clone(),
        None              => return Err("Error: Faied to acquire the repo information for the selected option.")
    }

    //Link repo.
    //NOTE: This is pretty hacky...
    status                = utils::StatusPrint::from_str(term, "Adding the repository to AppVeyor.");
    let     http_client   = Client::new();
    let mut response:       Response;
    let mut response_body = String::new();
    let     api_call_url:   Url;
    let     api_call      = format!("https://ci.appveyor.com/api/projects");
    let mut header        = Headers::new();
    let     auth          = format!("Bearer {}", config.appveyor_api_token);

    header.set_raw("Authorization",  vec![auth.into_bytes()]);
    header.set_raw("Content-Type",   vec![b"application/json".to_vec()]);

    match api_call.into_url() {
        Ok(url) => api_call_url = url,
        Err(_)  => {
            status.error(term);
            return Err("Error while parsing API call url.")
        }
    }

    let mut body: String = "{\"repositoryProvider\":\"gitHub\", \"repositoryName\":\"".to_string();
    body.push_str(&repo.full_name[..]);
    body.push_str("\"}");
    let body_len = body.len().clone();
    match http_client.post(api_call_url)
    .headers(header)
    .body(Body::BufBody(&body.into_bytes()[..], body_len))
    .send() {
        Ok(res) => response = res,
        Err(_)  => {
            status.error(term);
            return Err("Error calling the API.")
        }
    }

    match response.read_to_string(&mut response_body){
        Ok(_)  => (),
        Err(_) => {
            status.error(term);
            return Err("Error converting the API response to a string.")
        }
    }

    if response_body.contains("{\"message\":") {
        status.error(term);
    }

    status.success(term);
    *repo_tag = repo.full_name;
    Ok(())
}


pub fn encrypt_vars(board_info: &trello::TrelloBoardInfo, config: &config::TrelloBSTAPIConfig) -> Result<AppVeyorEncryptedVars, &'static str> {

    let mut enc_trello_app_token: String;
    match appveyor_encrypt_var(&config, &config.trello_app_token) {
        Ok(_enc_trello_app_token) => enc_trello_app_token = _enc_trello_app_token,
        Err(err) => return Err(err)
    }

    let mut enc_list_id: String;
    match appveyor_encrypt_var(&config, &board_info.list_id) {
        Ok(_enc_list_id) => enc_list_id = _enc_list_id,
        Err(err) => return Err(err)
    }

    let mut enc_build_pass_id: String;
    match appveyor_encrypt_var(&config, &board_info.build_pass_id) {
        Ok(_enc_build_pass_id) => enc_build_pass_id = _enc_build_pass_id,
        Err(err) => return Err(err)
    }

    let mut enc_build_fail_id: String;
    match appveyor_encrypt_var(&config, &board_info.build_fail_id) {
        Ok(_enc_build_fail_id) => enc_build_fail_id = _enc_build_fail_id,
        Err(err) => return Err(err)
    }

    Ok(AppVeyorEncryptedVars{
        trello_app_token: enc_trello_app_token,
        list_id:          enc_list_id,
        build_pass_id:    enc_build_pass_id,
        build_fail_id:    enc_build_fail_id
    })

}


pub fn appveyor_encrypt_var(config: &config::TrelloBSTAPIConfig, var: &String) -> Result<String, &'static str> {

    let     http_client   = Client::new();
    let mut response:       Response;
    let mut response_body = String::new();
    let     api_call_url:   Url;
    let     api_call      = format!("https://ci.appveyor.com/api/account/encrypt");
    let mut header        = Headers::new();
    let     auth          = format!("Bearer {}", config.appveyor_api_token);

    header.set_raw("Authorization",  vec![auth.into_bytes()]);
    header.set_raw("Content-Type",   vec![b"application/json;charset=utf-8".to_vec()]);

    match api_call.into_url() {
        Ok(url) => api_call_url = url,
        Err(_)  => {
            return Err("Error while parsing API call url.")
        }
    }

    let mut body: String = "{\"plainValue\":\"".to_string();
    body.push_str(&var[..]);
    body.push_str("\"}");
    ////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
    let body2 = body.clone();
    //let mut content_length: Vec<u8> = Vec::new();
    //content_length.push(81);
    //header.set_raw("Content-Length",   vec![content_length]);
////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
    let body_len = body.len().clone();
    match http_client.post(api_call_url)
    .headers(header)
    .body(Body::BufBody(&body.into_bytes()[..], body_len))
    .send() {
        Ok(res) => response = res,
        Err(_)  => {
            return Err("Error calling the API.")
        }
    }

    match response.read_to_string(&mut response_body){
        Ok(_)  => (),
        Err(_) => {
            return Err("Error converting the API response to a string.")
        }
    }

    ////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
    println!("\\\\\\\\\\\\\\\\\\\\\\{}", body2);
    println!("//////////////////////{}", response_body);
    if response_body.contains("{\"message\":") {
        return Err("Error encrypting variable.")
    }

    Ok(response_body)
}

pub fn generate_file(term: &mut Box<term::StdoutTerminal>, ci_config_output_dir: &PathBuf, encrypted_vars: &AppVeyorEncryptedVars) -> Result<(), &'static str> {

    let status                         = utils::StatusPrint::from_str(term, "Generating appveyor.yml");
    let mut appveyor_file:               File;
    let mut local_ci_config_output_dir = ci_config_output_dir.clone();
    local_ci_config_output_dir.push("appveyor.yml");
    match File::create(local_ci_config_output_dir.as_path()) {
        Ok(_appveyor_file)  => {
            appveyor_file = _appveyor_file;
        }
        Err(_)    => {
            status.error(term);
            return Err("Failed to create appveyor.yml");
        }
    }

    let mut file_data: String;
    file_data = format!("
environment:
  BUILD_DIRECTORY: ./
  COMPILER: MSVC
  TRELLO_API_KEY: {0}
  TRELLO_APP_TOKEN:
    secure: {1}
  TRELLO_API_LIST_ID:
    secure: {2}
  TRELLO_API_BUILD_PASS_ID:
    secure: {3}
  TRELLO_API_BUILD_FAIL_ID:
    secure: {4}

install:
before_build:
build_script:

on_success:
  - ps: |
    Remove-item alias:curl
    cd $($env:BUILD_DIRECTORY)
    7z a -r build.zip ./
    $buildLink       = [string](curl --silent --upload-file .\\build.zip https://transfer.sh/build.zip)
    $appveyor_branch = \"[$($env:APPVEYOR_REPO_BRANCH)]\"
    $ci_name         = \"[AppVeyor]\"
    $os_name         = \"[Windows]\"
    $compiler        = \"[$($env:COMPILER)]:%20\"
    $pass            = \"#$($env:APPVEYOR_BUILD_NUMBER)%20PASSED\"
    $card_name       = \"name=$($appveyor_branch)$($ci_name)$($os_name)$(compiler)$(pass)\"
    $additional_data = \"&due=null&pos=top\"
    $description     = \"&desc=\\[Build\\]:%20$($buildLink)%0D\\[Logs\\]:%20https://ci.appveyor.com/project/$($env:APPVEYOR_REPO_NAME)/build/$($env:APPVEYOR_BUILD_VERSION)/job/$($env:APPVEYOR_JOB_ID)
    $trello_data     = \"&idList=$($env:TRELLO_API_LIST_ID)&idLabels=$($env:TRELLO_API_BUILD_PASS_ID)&token=$($env:TRELLO_APP_TOKEN)&key=$($env:TRELLO_API_KEY)\"
    $data            = \"$($env:card_name)$($env:additional_data)$($env:description)$($env:trello_data)\"
    curl -s --data $($data) https://api.trello.com/1/cards > $null

  on_failure:
    - ps: |
    Remove-item alias:curl
    $appveyor_branch = \"[$($env:APPVEYOR_REPO_BRANCH)]\"
    $ci_name         = \"[AppVeyor]\"
    $os_name         = \"[Windows]\"
    $compiler        = \"[$($env:COMPILER)]:%20\"
    $pass            = \"#$($env:APPVEYOR_BUILD_NUMBER)%20PASSED\"
    $card_name       = \"name=$($appveyor_branch)$($ci_name)$($os_name)$(compiler)$(pass)\"
    $additional_data = \"&due=null&pos=top\"
    $description     = \"&desc=\\[Logs\\]:%20https://ci.appveyor.com/project/$($env:APPVEYOR_REPO_NAME)/build/$($env:APPVEYOR_BUILD_VERSION)/job/$($env:APPVEYOR_JOB_ID)
    $trello_data     = \"&idList=$($env:TRELLO_API_LIST_ID)&idLabels=$($env:TRELLO_API_BUILD_PASS_ID)&token=$($env:TRELLO_APP_TOKEN)&key=$($env:TRELLO_API_KEY)\"
    $data            = \"$($env:card_name)$($env:additional_data)$($env:description)$($env:trello_data)\"
    curl -s --data $($data) https://api.trello.com/1/cards > $null
", config::trello_api_key,
   encrypted_vars.trello_app_token,
   encrypted_vars.list_id,
   encrypted_vars.build_pass_id,
   encrypted_vars.build_fail_id);

    match appveyor_file.write_all(&file_data.into_bytes()[..]) {
        Ok(()) => (),
        Err(_) => {
            status.error(term);
            return Err("Error while writing to the file.");
        }
    }

    match appveyor_file.flush() {
        Ok(()) => (),
        Err(_) => {
            status.error(term);
            return Err("Error while flushing the file writing buffer.")
        }
    }

    status.success(term);
    Ok(())
}











