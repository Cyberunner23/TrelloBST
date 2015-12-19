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


use std::borrow::Borrow;
use std::error::Error;
use std::io;
use std::io::Read;

use config;

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

use utils;


////////////////////////////////////////////////////////////
//                         Macros                         //
////////////////////////////////////////////////////////////

include!("utils_macros.rs");


////////////////////////////////////////////////////////////
//                       Functions                        //
////////////////////////////////////////////////////////////

pub fn setup_api(term: &mut Box<term::StdoutTerminal>, is_using_config_file: bool, config: &mut config::TrelloBSTAPIConfig) -> Result<bool, &'static str> {

    if config.travis_access_token.is_empty() {

        //Get github token
        let mut github_token = String::new();
        get_input_string!(term, &mut github_token, "");
        if is_using_config_file {
            get_input_string!(term, &mut github_token, "Travis-CI currently uses a GitHub access token to authenticate and generate an API key,\n
                                                        please log into GitHub and go to https://github.com/settings/tokens and generate a new \n
                                                        token and input it here. (Note that once the Travis-CI API key is acquired, the GitHub \n
                                                        access token can be deleted.): ");
        } else {
            get_input_string!(term, &mut github_token, "Travis-CI currently uses a GitHub access token to authenticate and generate an API key,\n
                                                        please log into GitHub and go to https://github.com/settings/tokens and generate a new \n
                                                        token and input it here: ");
        }

        //Convert github token to travis api key
        let mut api_call = format!("https://api.travis-ci.org&github-token={}", github_token);
        let mut response_body = String::new();

        let mut header                  = Headers::new();
        let mut content_length: Vec<u8> = Vec::new();
        content_length.push(20 + github_token.len() as u8);
        header.set_raw("User-Agent",     vec![b"TrelloBST/0.0.1".to_vec()]);
        header.set_raw("Accept",         vec![b"application/vnd.travis-ci.2+json".to_vec()]);
        header.set_raw("Host",           vec![b"api.travis-ci.org".to_vec()]);
        header.set_raw("Content-Type",   vec![b"application/json".to_vec()]);
        header.set_raw("Content-Length", vec![content_length]);

        match utils::rest_api_call_post_with_header(&api_call, header) {
            Ok(_response_body) => response_body = _response_body,
            Err(err)           => return Err(err)
        }

        let data: Value;
        match serde_json::from_str(&response_body){
            Ok(_data) => data = _data,
            Err(_)    => return Err("Error parsing the response.")
        }

        //TODO: Better error reporting
        config.travis_access_token = data.as_object().unwrap().get("access_token").unwrap().as_string().unwrap().to_string();
    }

    Ok(is_using_config_file)
}

//pub fn create_travis_yml() -> Result<(), &'static str> {
//
//    //TODO: get pub key
//    //TODO: encrypt vars
//    //TODO: Generate file
//
//}



















