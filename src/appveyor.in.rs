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


pub fn create_appveyor_yml(term: &mut Box<term::StdoutTerminal>, config: &config::TrelloBSTAPIConfig, board_info: &mut trello::TrelloBoardInfo, ci_config_output_dir: &mut PathBuf) -> Result<(), &'static str>{

    //Select Repo
    let mut repo_tag = String::new();
    match repo_selection(term, config, &mut repo_tag) {
        Ok(())   => (),
        Err(err) => return Err(err)
    }

    //Encrypt Variables

    //Generate File

    Ok(())
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

    if response_body == "invalid key" {
        status.error(term);
        return Err("Error, the API key is invalid.");
    }

    if response_body == "invalid token" {
        status.error(term);
        return Err("The app token is invalid.");
    }

    if response_body.contains("{\"message\":") {
        status.error(term);
    }

    status.success(term);
    *repo_tag = repo.full_name;
    Ok(())
}



















