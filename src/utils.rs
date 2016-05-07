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
use std::fs;
use std::fs::OpenOptions;
use std::io::Read;
use std::path::PathBuf;

extern crate hyper;
use hyper::Client;
use hyper::client::IntoUrl;
use hyper::client::response::Response;
use hyper::header::Headers;
use hyper::Url;

extern crate serde;
extern crate serde_json;
use serde_json::Value;

extern crate term;

include!("utils_macros.rs");


////////////////////////////////////////////////////////////
//                     Structs & Impl                     //
////////////////////////////////////////////////////////////

pub struct StatusPrint {
    status_string: String
}

impl StatusPrint {

    #[allow(dead_code)]
    pub fn from_string(term: &mut Box<term::StdoutTerminal>, status: String) -> StatusPrint {
        match_to_none!(write!(term, "[ ] {}", status));
        match_to_none!(term.flush());
        StatusPrint{
            status_string: status,
        }
    }

    #[allow(dead_code)]
    pub fn from_str(term: &mut Box<term::StdoutTerminal>, status: &'static str) -> StatusPrint {
        match_to_none!(write!(term, "[ ] {}", status));
        match_to_none!(term.flush());
        StatusPrint{
            status_string: status.to_string(),
        }
    }

    #[allow(dead_code)]
    pub fn success(&self, term: &mut Box<term::StdoutTerminal>) {
        match_to_none!(term.carriage_return());
        match_to_none!(write!(term, "["));
        write_green!(term, "✓");
        match_to_none!(write!(term, "] {}\n", self.status_string));
        match_to_none!(term.flush());
    }

    #[allow(dead_code)]
    pub fn error(&self, term: &mut Box<term::StdoutTerminal>) {
        match_to_none!(term.carriage_return());
        match_to_none!(write!(term, "["));
        write_red!(term, "✗");
        match_to_none!(write!(term, "] {}\n", self.status_string));
        match_to_none!(term.flush());
    }
}



////////////////////////////////////////////////////////////
//                       Functions                        //
////////////////////////////////////////////////////////////

pub fn is_valid_dir(term: &mut Box<term::StdoutTerminal>, path: &PathBuf) -> bool {

    match fs::metadata(&path) {
        Ok(metadata) => {
            if !metadata.is_dir() {
                return false;
            }
        },
        Err(_)       => {
            writeln_red!(term, "Error: Failed to acquire directory's metadata.");
            return false;
        }
    }

    //Test if we can write a file to this directory
    let mut counter = 0;
    loop {
        //Generate a file name.
        let     tmp_file_name: String  = format!("ab{}ba.tmp", counter);
        let mut tmp_file_path: PathBuf = path.clone();
        tmp_file_path.push(tmp_file_name);

        //If file does not exist, check if we can create it with r/w permissions.
        if !tmp_file_path.exists() {
            if is_valid_file_path(&tmp_file_path) {
                match fs::remove_file(&tmp_file_path) {
                    Ok(_)  => (),
                    Err(_) => {writeln_red!(term, "Error: Failed to delete test file: {}", tmp_file_path.to_str().unwrap());}
                }
                break;
            } else {return false;}
        }
        counter += 1;
    }
    return true;
}

pub fn is_valid_file_path(path: &PathBuf) -> bool {
match OpenOptions::new().read(true).write(true).create(true).open(path.as_path()) {
        Ok(_)  => {return true;}
        Err(_) => {return false;}
    }
}

#[allow(dead_code)]
pub fn rest_api_call_get(api_call: &String) -> Result<String, &'static str> {

    let     http_client   = Client::new();
    let mut response:       Response;
    let mut response_body = String::new();
    let     api_call_url:   Url;

    match api_call.into_url() {
        Ok(url) => api_call_url = url,
        Err(_)  => return Err("Error while parsing API call url.")
    }

    match http_client.get(api_call_url).send() {
        Ok(res) => response = res,
        Err(_)  => return Err("Error calling the API.")
    }

    match response.read_to_string(&mut response_body) {
        Ok(_)  => (),
        Err(_) => return Err("Error converting the API response to a string.")
    }

    if response_body == "invalid key" {
        return Err("Error, the API key is invalid.");
    }

    if response_body == "invalid token" {
        return Err("The app token is invalid.");
    }

    Ok(response_body)
}

#[allow(dead_code)]
pub fn rest_api_call_get_with_header(api_call: &String, header: Headers) -> Result<String, &'static str> {

    let     http_client   = Client::new();
    let mut response:       Response;
    let mut response_body = String::new();
    let     api_call_url:   Url;

    match api_call.into_url() {
        Ok(url) => api_call_url = url,
        Err(_)  => return Err("Error while parsing API call url.")
    }

    match http_client.get(api_call_url)
                     .headers(header)
                     .send() {
        Ok(res) => response = res,
        Err(_)  => {
            return Err("Error calling the API.");
        }
    }

    match response.read_to_string(&mut response_body) {
        Ok(_)  => (),
        Err(_) => return Err("Error converting the API response to a string.")
    }

    if response_body == "invalid key" {
        return Err("Error, the API key is invalid.");
    }

    if response_body == "invalid token" {
        return Err("The app token is invalid.");
    }

    Ok(response_body)
}

#[allow(dead_code)]
pub fn rest_api_call_post(api_call: &String) -> Result<String, &'static str> {

    let     http_client   = Client::new();
    let mut response:       Response;
    let mut response_body = String::new();
    let     api_call_url:   Url;

    match api_call.into_url() {
        Ok(url) => api_call_url = url,
        Err(_)  => return Err("Error while parsing API call url.")
    }

    match http_client.post(api_call_url).send() {
        Ok(res) => response = res,
        Err(_)  => return Err("Error calling the API.")
    }

    match response.read_to_string(&mut response_body){
        Ok(_)  => (),
        Err(_) => return Err("Error converting the API response to a string.")
    }

    if response_body == "invalid key" {
        return Err("Error, the API key is invalid.");
    }

    if response_body == "invalid token" {
        return Err("The app token is invalid.");
    }

    Ok(response_body)
}

#[allow(dead_code)]
pub fn rest_api_call_post_with_header(api_call: &String, header: Headers) -> Result<String, &'static str> {

    let     http_client   = Client::new();
    let mut response:       Response;
    let mut response_body = String::new();
    let     api_call_url:   Url;

    match api_call.into_url() {
        Ok(url) => api_call_url = url,
        Err(_)  => return Err("Error while parsing API call url.")
    }

    match http_client.post(api_call_url)
            .headers(header)
            .send() {
        Ok(res) => response = res,
        Err(_)  => return Err("Error calling the API.")
    }

    match response.read_to_string(&mut response_body){
        Ok(_)  => (),
        Err(_) => return Err("Error converting the API response to a string.")
    }

    if response_body == "invalid key" {
        return Err("Error, the API key is invalid.");
    }

    if response_body == "invalid token" {
        return Err("The app token is invalid.");
    }

    Ok(response_body)
}

#[allow(dead_code)]
pub fn rest_api_call_put_with_header(api_call: &String, header: Headers) -> Result<String, &'static str> {

    let     http_client   = Client::new();
    let mut response:       Response;
    let mut response_body = String::new();
    let     api_call_url:   Url;

    match api_call.into_url() {
        Ok(url) => api_call_url = url,
        Err(_)  => return Err("Error while parsing API call url.")
    }

    match http_client.put(api_call_url)
                     .headers(header)
                     .send() {
        Ok(res) => response = res,
        Err(_)  => return Err("Error calling the API.")
    }

    match response.read_to_string(&mut response_body){
        Ok(_)  => (),
        Err(_) => return Err("Error converting the API response to a string.")
    }

    if response_body == "invalid key" {
        return Err("Error, the API key is invalid.");
    }

    if response_body == "invalid token" {
        return Err("The app token is invalid.");
    }

    Ok(response_body)
}

#[allow(dead_code)]
pub fn get_single_json_value_as_string(json_string: &String, field: &str) -> Result<String, &'static str>{

    let data: Value;
    match serde_json::from_str(&json_string){
        Ok(_data) => data = _data,
        Err(_)  => {
            return Err("Error parsing the JSON data")
        }
    }

    let object: BTreeMap<String, Value>;
    match data.as_object().ok_or("Error: JSON data does not describe an object.") {
        Ok(_object) => {
            object  = _object.clone();
        },
        Err(err)    => {
            return Err(err);
        }
    }

    let json_value: Value;
    match object.get(field).ok_or("Error: The field has not been found in the JSON data.") {
        Ok(_json_value) => {
            json_value  = _json_value.clone();
        }
        Err(err)        => {
            return Err(err)
        }
    }

    match json_value.as_string().ok_or("Error: The field's value is not a string.") {
        Ok(_value) => Ok(_value.to_string()),
        Err(err)   => Err(err)
    }
}




































