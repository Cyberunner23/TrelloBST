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
use std::io::{self, Read, Write};
use std::path::PathBuf;
use std::process::exit;

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

//Status printing
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


//Menu builder
pub struct MenuBuilderItem<T> {
    entry_name: String,
    object:     T
}

pub struct MenuBuilder<T> {
    menu_items:        BTreeMap<usize, MenuBuilderItem<T>>,
    menu_item_counter: usize,
    display_msg:       String
}

impl<T> MenuBuilder<T> {

    pub fn new(display_message: String) -> MenuBuilder<T> {
        return MenuBuilder {
            menu_items:        BTreeMap::new(),
            menu_item_counter: 0,
            display_msg:       display_message
        } as MenuBuilder<T>;
    }

    pub fn add_entry(&mut self, name: String, entry_object: T) {
        let menu_item = MenuBuilderItem {
            entry_name: name,
            object: entry_object
        };
        self.menu_item_counter += 1;
        self.menu_items.insert(self.menu_item_counter, menu_item);
    }

    pub fn select(&mut self, term: &mut Box<term::StdoutTerminal>) -> &mut T {

        //Print options
        println!("{}\n", self.display_msg);
        for (entry_number, object) in self.menu_items.iter_mut() {
            println!("[{}]: {}", entry_number, object.entry_name);
        }
        writeln_red!(term, "[0]: Quit");

        //Get input.
        let mut option: usize = 0;
        loop {
            get_input_usize!(term, &mut option, "Please enter an option: ");
            if option <= self.menu_items.len(){
                break;
            }else {
                writeln_red!(term, "Please enter a valid option.");
            }
        }

        //Return object according to input
        if option == 0 {
            exit(0);
        } else {
             match self.menu_items.get_mut(&option) {
                 Some(obj) => {
                     return &mut obj.object;
                 }
                 //Panick on None (something baaaaad happened if we get his)
                 None      => {
                     panic!("Menu entry missing, this is a bug...");
                 }
             }
        }
    }
}


////////////////////////////////////////////////////////////
//                       Functions                        //
////////////////////////////////////////////////////////////

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




































