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

use std::io::Read;

extern crate hyper;
use hyper::Client;
use hyper::client::IntoUrl;
use hyper::client::response::Response;
use hyper::header::Headers;
use hyper::Url;

extern crate term;

include!("utils_macros.rs");


////////////////////////////////////////////////////////////
//                     Structs & Impl                     //
////////////////////////////////////////////////////////////

pub struct StatusPrint {
    status_string: String
}

impl StatusPrint {
    pub fn from_string(term: &mut Box<term::StdoutTerminal>, status: String) -> StatusPrint {
        match_to_none!(write!(term, "[ ] {}", status));
        match_to_none!(term.flush());
        StatusPrint{
            status_string: status,
        }
    }

    pub fn from_str(term: &mut Box<term::StdoutTerminal>, status: &'static str) -> StatusPrint {
        match_to_none!(write!(term, "[ ] {}", status));
        match_to_none!(term.flush());
        StatusPrint{
            status_string: status.to_string(),
        }
    }

    pub fn success(&self, term: &mut Box<term::StdoutTerminal>) {
        match_to_none!(term.carriage_return());
        match_to_none!(write!(term, "["));
        write_green!(term, "✓");
        match_to_none!(write!(term, "] {}\n", self.status_string));
        match_to_none!(term.flush());
    }

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

pub fn rest_api_call_get(api_call: &String) -> Result<String, &'static str> {

    let     http_client   = Client::new();
    let mut response:       Response;
    let mut response_body = String::new();
    let mut api_call_url:   Url;

    match api_call.into_url() {
        Ok(url) => api_call_url = url,
        Err(_)  => return Err("Error while parsing API call url.")
    }

    match http_client.get(api_call_url).send() {
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

pub fn rest_api_call_post(api_call: &String) -> Result<String, &'static str> {

    let     http_client   = Client::new();
    let mut response:       Response;
    let mut response_body = String::new();
    let mut api_call_url:   Url;

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

pub fn rest_api_call_post_with_header(api_call: &String, header: Headers) -> Result<String, &'static str> {

    let     http_client   = Client::new();
    let mut response:       Response;
    let mut response_body = String::new();
    let mut api_call_url:   Url;

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







