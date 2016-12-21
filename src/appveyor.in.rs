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
use std::io::Write;
use std::fs::File;
use std::path::PathBuf;

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
    description:         Option<String>,
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
    pub trello_api_token: String,
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
            description:         Some(String::new()),
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

    pub fn from_api_call(&mut self, config: &mut config::TrelloBSTConfig) -> Result<(), &'static str> {

        //Do API call.
        let     api_call      = format!("https://ci.appveyor.com/api/repositories/gitHub");
        let mut header        = Headers::new();
        let     auth          = format!("Bearer {}", config.get("appveyor_api_token"));

        header.set_raw("Authorization",  vec![auth.into_bytes()]);
        header.set_raw("Content-Type",   vec![b"application/json".to_vec()]);

        let response_body = try!(utils::rest_api_call_get_with_header(&api_call, header));

        //Parse raw array.
        let data: Value = match serde_json::from_str(&response_body){
            Ok(data) => data,
            Err(_)   => {return Err("Error parsing the JSON data");}
        };

        //Get group Info.
        let group_info_array: Vec<Value> = try!(data.as_array().ok_or("Error: The JSON response from GithubResponse is not an array.")).clone();

        for group in group_info_array {

            let group_info = try!(group.as_object().ok_or("Error: Expected an array of JSON objects in GithubResponse.")).clone();

            let group_type_value = try!(group_info.get("groupType").ok_or("Error: Could not find the \"groupType\" field in a GithubResponse object."));
            let group_type       = try!(group_type_value.as_str().ok_or("Error: Failed to parse the value of \"groupType\" in the GithubResponse object.")).to_string();

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
//                        AppVeyor                        //
////////////////////////////////////////////////////////////

#[derive(Clone)]
pub struct AppVeyor{}

impl CITrait for AppVeyor {
    fn get_filename(&mut self) -> String {return "appveyor.yml".to_string();}

    fn get_name(&mut self) -> String {return "AppVeyor".to_string();}

    fn setup(&mut self, term: &mut Box<term::StdoutTerminal>, config: &mut config::TrelloBSTConfig) -> Result<(), String> {

        //API key
        if config.get("appveyor_api_token").is_empty() {
            //Get appveyo api key
            let mut key: String = String::new();
            get_input_string!(term, &mut key, "TrelloBST currently supports repos on github only.
    Please log into appveyor and link your github account then go to https://ci.appveyor.com/api-token and enter
    your api token here: ");
            config.set("appveyor_api_token", &key[..]);
        }

        Ok(())
    }

    fn generate_ci_config(&mut self, term: &mut Box<term::StdoutTerminal>, config: &mut config::TrelloBSTConfig) -> Result<(String, String), String> {

        //Select Repo
        let mut repo_tag = String::new();
        match self.repo_selection(term, config, &mut repo_tag) {
            Ok(())   => (),
            Err(err) => return Err(err)
        }

        //Encrypt Variables
        let status         = utils::StatusPrint::from_str(term, "Encrypting Trello API values.");
        let encrypted_vars = match self.encrypt_vars(config) {
            Ok(vars) => {
                status.success(term);
                vars
            },
            Err(err) => {
                status.error(term);
                return Err(err)
            }
        };

        //Generate File
        let mut file_data = include_str!("templates/appveyor").to_string();;

        file_data = file_data.replace("<TRELLO_API_TOKEN>",         &encrypted_vars.trello_api_token[..]);
        file_data = file_data.replace("<TRELLO_API_LIST_ID>",       &encrypted_vars.list_id[..]);
        file_data = file_data.replace("<TRELLO_API_BUILD_PASS_ID>", &encrypted_vars.build_pass_id[..]);
        file_data = file_data.replace("<TRELLO_API_BUILD_FAIL_ID>", &encrypted_vars.build_fail_id[..]);

        Ok((self.get_filename(), file_data))
    }
}

impl AppVeyor {

    pub fn appveyor_encrypt_var(&mut self, config: &mut config::TrelloBSTConfig, var: &str) -> Result<String, String> {

        let     http_client   = Client::new();
        let mut response_body = String::new();
        let     api_call      = format!("https://ci.appveyor.com/api/account/encrypt");
        let mut header        = Headers::new();
        let     auth          = format!("Bearer {}", config.get("appveyor_api_token"));

        header.set_raw("Authorization",  vec![auth.into_bytes()]);
        header.set_raw("Content-Type",   vec![b"application/json;charset=utf-8".to_vec()]);

        let api_call_url = match api_call.into_url() {
            Ok(url) => url,
            Err(_)  => {return Err("Error while parsing API call url.".to_string());}
        };

        let mut body: String = "{\"plainValue\":\"".to_string();
        body.push_str(&config.get(var)[..]);
        body.push_str("\"}");

        let body_len = body.len().clone();
        let mut response = match http_client.post(api_call_url)
            .headers(header)
            .body(Body::BufBody(&body.into_bytes()[..], body_len))
            .send() {
            Ok(res) => res,
            Err(_)  => {return Err("Error calling the API.".to_string());}
        };

        match response.read_to_string(&mut response_body){
            Ok(_)  => (),
            Err(_) => {return Err("Error converting the API response to a string.".to_string());}
        }

        if response_body.contains("{\"message\":") {
            return Err("Error encrypting variable.".to_string())
        }

        Ok(response_body)
    }

    pub fn encrypt_vars(&mut self, config: &mut config::TrelloBSTConfig) -> Result<AppVeyorEncryptedVars, String> {
        Ok(AppVeyorEncryptedVars{
            trello_api_token: try!(self.appveyor_encrypt_var(config, "trello_api_token")),
            list_id:          try!(self.appveyor_encrypt_var(config, "trello_list_id")),
            build_pass_id:    try!(self.appveyor_encrypt_var(config, "trello_label_pass_id")),
            build_fail_id:    try!(self.appveyor_encrypt_var(config, "trello_label_fail_id"))
        })
    }

    pub fn repo_selection(&mut self, term: &mut Box<term::StdoutTerminal>, config: &mut config::TrelloBSTConfig, repo_tag: &mut String) -> Result<(), String> {

        let mut status      = utils::StatusPrint::from_str(term, "Acquiring the repo list from AppVeyor.");
        let mut groups_info = GithubResponse::new();
        match groups_info.from_api_call(config) {
            Ok(())   => status.success(term),
            Err(err) => {
                status.error(term);
                return Err(err.to_string())
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
        let mut repo = try!(repos.get(&option).ok_or("Error: Faied to acquire the repo information for the selected option.")).clone();

        //Link repo.
        //NOTE: This is pretty hacky...
        status                = utils::StatusPrint::from_str(term, "Adding the repository to AppVeyor.");
        let     http_client   = Client::new();
        let mut response_body = String::new();
        let     api_call      = format!("https://ci.appveyor.com/api/projects");
        let mut header        = Headers::new();
        let     auth          = format!("Bearer {}", config.get("appveyor_api_token"));

        header.set_raw("Authorization",  vec![auth.into_bytes()]);
        header.set_raw("Content-Type",   vec![b"application/json".to_vec()]);

        let api_call_url = match api_call.into_url() {
            Ok(url) => url,
            Err(_)  => {
                status.error(term);
                return Err("Error while parsing API call url.".to_string())
            }
        };

        let mut body: String = "{\"repositoryProvider\":\"gitHub\", \"repositoryName\":\"".to_string();
        body.push_str(&repo.full_name[..]);
        body.push_str("\"}");

        let     body_len = body.len().clone();
        let mut response = match http_client.post(api_call_url)
            .headers(header)
            .body(Body::BufBody(&body.into_bytes()[..], body_len))
            .send() {
            Ok(res) => res,
            Err(_)  => {
                status.error(term);
                return Err("Error calling the API.".to_string())
            }
        };

        match response.read_to_string(&mut response_body){
            Ok(_)  => (),
            Err(_) => {
                status.error(term);
                return Err("Error converting the API response to a string.".to_string())
            }
        }

        if response_body.contains("{\"message\":") {
            status.error(term);
        }

        status.success(term);
        *repo_tag = repo.full_name;
        Ok(())
    }

}
