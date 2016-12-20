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

use std::env;

extern crate hyper;
use hyper::header::Headers;

extern crate url;
use self::url::percent_encoding;

use utils;

////////////////////////////////////////////////////////////
//                        Structs                         //
////////////////////////////////////////////////////////////

pub struct PushConfig {
    pub trello_api_token:         String,
    pub trello_api_list_id:       String,
    pub trello_api_build_pass_id: String,
    pub trello_api_build_fail_id: String,
    pub card_title:               String,
    pub card_desc:                String,
}


////////////////////////////////////////////////////////////
//                         Impls                          //
////////////////////////////////////////////////////////////

impl PushConfig {
    pub fn fill(cli_card_title:    String,
                cli_card_desc:     String,
                cli_build_pass_id: String,
                cli_build_fail_id: String,
                cli_list_id:       String,
                cli_api_token:     String) -> Result<PushConfig, &'static str> {

        let mut tmp_trello_api_token         = String::new();
        let mut tmp_trello_api_list_id       = String::new();
        let mut tmp_trello_api_build_pass_id = String::new();
        let mut tmp_trello_api_build_fail_id = String::new();

        //NOTE: No need to check card title and card desc

        //Get build pass id from env var (if not given as a cli option, fail if not in env var)
        if cli_build_pass_id.is_empty() {
            //If empty check env var.
            match env::var("TRELLO_API_BUILD_PASS_ID") {
                Ok(val)  => {tmp_trello_api_build_pass_id = val;}
                Err(_) => {return Err("Error getting the \"TRELLO_API_BUILD_PASS_ID\" environment variable, either undefined or value not in Unicode");}
            }
        } else {
            tmp_trello_api_build_pass_id = cli_build_pass_id;
        }

        //Get build fail id from env var (if not given as a cli option, fail if not in env var)
        if cli_build_fail_id.is_empty() {
            //If empty check env var.
            match env::var("TRELLO_API_BUILD_FAIL_ID") {
                Ok(val)  => {tmp_trello_api_build_fail_id = val;}
                Err(_) => {return Err("Error getting the \"TRELLO_API_BUILD_FAIL_ID\" environment variable, either undefined or value not in Unicode");}
            }
        } else {
            tmp_trello_api_build_fail_id = cli_build_fail_id;
        }

        //Get card list id from env var (if not given as a cli option, fail if not in env var)
        if cli_list_id.is_empty() {
            //If empty check env var.
            match env::var("TRELLO_API_LIST_ID") {
                Ok(val)  => {tmp_trello_api_list_id = val;}
                Err(_) => {return Err("Error getting the \"TRELLO_API_LIST_ID\" environment variable, either undefined or value not in Unicode");}
            }
        } else {
            tmp_trello_api_list_id = cli_list_id;
        }

        //Get api token from env var (if not given as a cli option, fail if not in env var)
        if cli_api_token.is_empty() {
            //If empty check env var.
            match env::var("TRELLO_API_TOKEN") {
                Ok(val)  => {tmp_trello_api_token = val;}
                Err(_) => {return Err("Error getting the \"TRELLO_API_TOKEN\" environment variable, either undefined or value not in Unicode");}
            }
        } else {
            tmp_trello_api_token = cli_api_token;
        }

        Ok(PushConfig {
            trello_api_token:         tmp_trello_api_token,
            trello_api_list_id:       tmp_trello_api_list_id,
            trello_api_build_pass_id: tmp_trello_api_build_pass_id,
            trello_api_build_fail_id: tmp_trello_api_build_fail_id,
            card_title:               cli_card_title,
            card_desc:                cli_card_desc
        })
    }
}


////////////////////////////////////////////////////////////
//                       Functions                        //
////////////////////////////////////////////////////////////

pub fn push(api_key: String, is_pass: bool, push_data: PushConfig) -> Result<(), &'static str> {

    //Setup push packet header.

    let label: String;
    if is_pass {
        label = push_data.trello_api_build_pass_id;
    } else {
        label = push_data.trello_api_build_fail_id;
    }

    let mut card_title: String = percent_encoding::percent_encode(push_data.card_title.as_bytes(), percent_encoding::USERINFO_ENCODE_SET).collect();
    let mut card_desc:  String = percent_encoding::percent_encode(push_data.card_desc.as_bytes(),  percent_encoding::USERINFO_ENCODE_SET).collect();

    let api_call = format!("https://api.trello.com/1/cards?key={}&token={}&idList={}&name={}&desc={}&idLabels={}&due=null&pos=top",
                           api_key,
                           push_data.trello_api_token,
                           push_data.trello_api_list_id,
                           card_title,
                           card_desc,
                           label);
    let mut response_body = String::new();
    let mut header        = Headers::new();

    //Send off the packet
    match utils::rest_api_call_post_with_header(&api_call, header) {
        Ok(_response_body) => {
            response_body = _response_body;
            Ok(())
        }
        Err(err)           => {
            Err(err)
        }
    }
}
































