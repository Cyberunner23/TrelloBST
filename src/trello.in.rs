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

//TODO: put api calling in a function or macro

use std::error::Error;
use std::io;
use std::process::exit;

use config;

use serde_json::Value;

extern crate term;

mod utils;


////////////////////////////////////////////////////////////
//                         Macros                         //
////////////////////////////////////////////////////////////

include!("utils_macros.rs");


////////////////////////////////////////////////////////////
//                        Structs                         //
////////////////////////////////////////////////////////////

pub struct TrelloBoardInfo {
    pub board_id:      String,
    pub list_id:       String,
    pub build_pass_id: String,
    pub build_fail_id: String,
}


#[derive(Deserialize)]
pub struct BoardNameID {
    pub name: String,
    pub id:   String
}

//response to the /1/boards/[idBoard] api call.
#[derive(Deserialize)]
pub struct MembersMeBoardsResponse {
    pub id:     String,
    pub boards: Vec<BoardNameID>,
}


#[derive(Deserialize)]
pub struct ListInfo {
    pub id:   String,
    pub name: String
}

//response to the /1/boards api call.
#[derive(Deserialize)]
pub struct BoardsResponse {
    pub id:    String,
    pub name:  String,
    pub desc:  String,
    pub lists: Vec<ListInfo>
}

#[derive(Deserialize)]
pub struct LabelInfo {
    pub id:    String,
    pub name:  String,
    pub color: String
}

#[derive(Deserialize)]
pub struct BoardsLabelsResponse {
    pub id:     String,
    pub labels: Vec<LabelInfo>
}

////////////////////////////////////////////////////////////
//                         Impls                          //
////////////////////////////////////////////////////////////

impl TrelloBoardInfo {
    pub fn new() -> TrelloBoardInfo {
        TrelloBoardInfo {
            board_id:      String::new(),
            list_id:       String::new(),
            build_pass_id: String::new(),
            build_fail_id: String::new(),
        }
    }
}


impl MembersMeBoardsResponse{
    pub fn new() -> MembersMeBoardsResponse{
        MembersMeBoardsResponse{
            id:     String::new(),
            boards: Vec::new(),
        }
    }
}


impl BoardsResponse {
    pub fn new() -> BoardsResponse{
        BoardsResponse {
            id:    String::new(),
            name:  String::new(),
            desc:  String::new(),
            lists: Vec::new(),
        }
    }
}


impl BoardsLabelsResponse {
    pub fn new() -> BoardsLabelsResponse {
        BoardsLabelsResponse{
            id:     String::new(),
            labels: Vec::new(),
        }
    }
}


////////////////////////////////////////////////////////////
//                       Functions                        //
////////////////////////////////////////////////////////////

pub fn setup_api(term: &mut Box<term::StdoutTerminal>, config: &mut config::TrelloBSTAPIConfig) {
    if config.trello_app_token.is_empty(){
        println!("Setting up Trello API Token...");
        get_input_string!(term, &mut config.trello_app_token, "Log in to Trello.com and enter the app token from https://trello.com/1/authorize?response_type=token&key=0e190833c4db5fd7d3b0b26ae642d6fa&scope=read%2Cwrite&expiration=never&name=TrelloBST : ");
    }
}


pub fn setup_board(term: &mut Box<term::StdoutTerminal>, config: &mut config::TrelloBSTAPIConfig, board_info: &mut TrelloBoardInfo) -> bool{

    //Aquire board list
    let mut board_list = MembersMeBoardsResponse::new();
    let     status     = utils::StatusPrint::from_str(term, "Acquiring board list from Trello.");
    match acquire_board_list(&config, &mut board_list) {
        Ok(_)    => {
            status.success(term);
        },
        Err(err) => {
            status.error(term);
            panic!(format!("An error occurred while communicating with Trello: {}", err));
        },
    }

    //Board selection.
    board_selection(term, config, &mut board_list, board_info)
}

#[allow(unused_assignments)]
pub fn acquire_board_list(config: &config::TrelloBSTAPIConfig, board_list: &mut MembersMeBoardsResponse) -> Result<(), &'static str>{

    let     api_call      = format!("https://api.trello.com/1/members/me?fields=&boards=open&board_fields=name&key={}&token={}", config::trello_api_key, config.trello_app_token);
    let mut response_body = String::new();

    match utils::rest_api_call_get(&api_call) {
        Ok(_response_body) => response_body = _response_body,
        Err(err)           => return Err(err)
    }

    let local_board_list: MembersMeBoardsResponse;
    match serde_json::from_str(&response_body){
        Ok(_board_list) => local_board_list = _board_list,
        Err(_)          => return Err("Error parsing the response.")
    }

    board_list.id     = local_board_list.id;
    board_list.boards = local_board_list.boards;

    Ok(())
}

pub fn board_selection(term: &mut Box<term::StdoutTerminal>, config: &mut config::TrelloBSTAPIConfig, board_list: &mut MembersMeBoardsResponse, board_info: &mut TrelloBoardInfo) -> bool {

    println!("Which board do you want to setup?");
    let mut counter = 1;
    for i in 0..board_list.boards.len() {
        println!("[{}] {}", i + 1, board_list.boards[i].name);
        counter += 1;
    }
    writeln_green!(term, "[{}] Create a new board.", counter);
    writeln_red!(term, "[0] Quit.");

    let mut option: usize = 0;
    loop {

        get_input_usize!(term, &mut option, "Please enter an option: ");

        if option <= counter && option >= 0 {
            break;
        }else {
            writeln_red!(term, "Please enter a valid option.");
        }
    }

    let mut is_board_created = false;
    if option == counter {
        match create_board(term, &config, board_info){
            Ok(_)    => is_board_created = true,
            Err(err) => {
                panic!(format!("An error occured: {}", err));
            }
        }
    } else if option == 0 {
        exit(0)
    } else {
        board_info.board_id = board_list.boards[option - 1].id.clone();
    }

    return is_board_created;
}

#[allow(unused_assignments)]
pub fn create_board(term: &mut Box<term::StdoutTerminal>, config: &config::TrelloBSTAPIConfig, board_info: &mut TrelloBoardInfo) -> Result<(), &'static str> {

    //Create board
    let mut board_name             = String::new();
    let mut is_input_success: bool = false;
    loop {
        get_input_string_success!(term, &mut board_name, &mut is_input_success, "Please enter a name for the new board: ");
        if is_input_success {break;}
    }

    let     status = utils::StatusPrint::from_str(term, "Creating the board.");
    let     api_call      = format!("https://trello.com/1/boards?name={}&defaultLists=false&key={}&token={}", board_name, config::trello_api_key, config.trello_app_token);
    let mut response_body = String::new();

    match utils::rest_api_call_post(&api_call) {
        Ok(_response_body) => response_body = _response_body,
        Err(err)           => {
            status.error(term);
            return Err(err);
        }
    }

    match utils::get_single_json_value_as_string(&response_body, "id") {
        Ok(value) => board_info.board_id = value,
        Err(err)  => {
            status.error(term);
            return Err(err)
        }
    }

    status.success(term);
    Ok(())
}


pub fn setup_list(term: &mut Box<term::StdoutTerminal>, config: &mut config::TrelloBSTAPIConfig, board_info: &mut TrelloBoardInfo, is_board_created: &bool) {

    if !*is_board_created {

        //If the board wasn't just created, acquire list
        let     status           = utils::StatusPrint::from_str(term, "Acquiring board's lists list from Trello.");
        let mut board_lists_list = BoardsResponse::new();
        match acquire_board_lists_list(&config, &board_info, &mut board_lists_list) {
            Ok(_)    => {
                status.success(term);
            },
            Err(err) => {
                status.error(term);
                panic!(format!("An error occurred while communicating with Trello: {}", err));
            },
        }

        //And list selection
        board_list_selection(term, config, &mut board_lists_list, board_info);

    } else {
        //If the board was just created then create the list also.
        match create_list(term, &config, board_info){
            Ok(_)    => (),
            Err(err) => {
                panic!(format!("An error occured: {}", err));
            }
        }
    }
}

#[allow(unused_assignments)]
pub fn acquire_board_lists_list(config: &config::TrelloBSTAPIConfig, board_info: &TrelloBoardInfo, board_lists_list: &mut BoardsResponse) -> Result<(), &'static str>{

    let api_call      = format!("https://api.trello.com/1/boards/{}?lists=open&list_fields=name&fields=name,desc&key={}&token={}",board_info.board_id, config::trello_api_key, config.trello_app_token);
    let mut response_body = String::new();

    match utils::rest_api_call_get(&api_call) {
        Ok(_response_body) => response_body = _response_body,
        Err(err)           => return Err(err)
    }

    let local_board_lists_list: BoardsResponse;
    match serde_json::from_str(&response_body){
        Ok(_board_lists_list) => local_board_lists_list = _board_lists_list,
        Err(_)                => return Err("Error parsing the response.",)
    }

    board_lists_list.id    = local_board_lists_list.id;
    board_lists_list.name  = local_board_lists_list.name;
    board_lists_list.desc  = local_board_lists_list.desc;
    board_lists_list.lists = local_board_lists_list.lists;

    Ok(())

}

pub fn board_list_selection(term: &mut Box<term::StdoutTerminal>, config: &mut config::TrelloBSTAPIConfig, board_lists_list: &mut BoardsResponse, board_info: &mut TrelloBoardInfo) {

    println!("Which board list do you want to use for the build statuses?");

    let mut counter = 1;
    for i in 0..board_lists_list.lists.len() {
        println!("[{}] {}", i + 1, board_lists_list.lists[i].name);
        counter += 1;
    }
    writeln_green!(term, "[{}] Create a new list.", counter);
    writeln_red!(term,   "[0] Quit.");


    let mut option: usize = 0;
    loop {

        get_input_usize!(term, &mut option, "Please enter an option: ");

        if option <= counter && option >= 0 {
            break;
        }else {
            writeln_red!(term, "Please enter a valid option.");
        }
    }

    if option == counter {
        match create_list(term, &config, board_info){
            Ok(_)    => (),
            Err(err) => {
                panic!(format!("An error occured: {}", err));
            }
        }
    } else if option == 0 {
        exit(0)
    } else {
        board_info.list_id = board_lists_list.lists[option - 1].id.clone();
    }
}

#[allow(unused_assignments)]
pub fn create_list(term: &mut Box<term::StdoutTerminal>, config: &config::TrelloBSTAPIConfig, board_info: &mut TrelloBoardInfo) -> Result<(), &'static str> {

    let mut list_name        = String::new();
    let mut is_input_success = false;
    loop {
        get_input_string_success!(term, &mut list_name, &mut is_input_success, "Please enter a name for the list which will contain the build statuses: ");
        if is_input_success {break;}
    }

    let status            = utils::StatusPrint::from_str(term, "Creating the list.");
    let api_call          = format!("https://trello.com/1/lists?name={}&idBoard={}&defaultLists=false&key={}&token={}", list_name, board_info.board_id, config::trello_api_key, config.trello_app_token);
    let mut response_body = String::new();

    match utils::rest_api_call_post(&api_call) {
        Ok(_response_body) => response_body = _response_body,
        Err(err)           => {
            status.error(term);
            return Err(err);
        }
    }

    match utils::get_single_json_value_as_string(&response_body, "id") {
        Ok(value) => board_info.list_id = value,
        Err(err)  => {
            status.error(term);
            return Err(err);
        }
    }

    status.success(term);
    Ok(())
}


pub fn setup_labels(term: &mut Box<term::StdoutTerminal>, config: &mut config::TrelloBSTAPIConfig, board_info: &mut TrelloBoardInfo, is_board_created: &bool) {

    if !*is_board_created {

        //If the board wasn't just created, acquire list
        let     status           = utils::StatusPrint::from_str(term, "Acquiring board's labels from Trello.");
        let mut board_label_list = BoardsLabelsResponse::new();
        match acquire_board_label_list(&config, &board_info, &mut board_label_list) {
            Ok(_)    => {
                status.success(term);
            },
            Err(err) => {
                status.error(term);
                panic!(format!("An error occurred while communicating with Trello: {}", err));
            },
        }

        //And label selection
        //Build pass
        board_label_selection_pass(term, config, &mut board_label_list, board_info);
        //Build fail
        board_label_selection_fail(term, config, &mut board_label_list, board_info);

    } else {
        //If the board was just created then create the list also.
        match create_label_pass(term, &config, board_info){//
            Ok(_)    => (),//
            Err(err) => {
                panic!(format!("An error occured: {}", err));
            }
        }

        match create_label_fail(term, &config, board_info){//
            Ok(_)    => (),//
            Err(err) => {
                panic!(format!("An error occured: {}", err));
            }
        }
    }

}

//TODO?: Create a manual parser.
pub fn acquire_board_label_list(config: &config::TrelloBSTAPIConfig, board_info: &TrelloBoardInfo, board_label_list: &mut BoardsLabelsResponse) -> Result<(), &'static str> {

    let     api_call       = format!("https://api.trello.com/1/boards/{}?labels=all&label_fields=name,color&fields=none&key={}&token={}",board_info.board_id, config::trello_api_key, config.trello_app_token);
    let mut response_body = String::new();

    match utils::rest_api_call_get(&api_call) {
        Ok(_response_body) => response_body = _response_body,
        Err(err)           => return Err(err)
    }

    let local_board_lebels_response: BoardsLabelsResponse;
    match serde_json::from_str(&response_body){
        Ok(_board_lists_list) => local_board_lebels_response = _board_lists_list,
        Err(_)                => return Err("Error parsing the response.",)
    }

    board_label_list.id     = local_board_lebels_response.id;
    board_label_list.labels = local_board_lebels_response.labels;

    Ok(())
}

pub fn board_label_selection_pass(term: &mut Box<term::StdoutTerminal>, config: &mut config::TrelloBSTAPIConfig, board_label_list: &mut BoardsLabelsResponse, board_info: &mut TrelloBoardInfo) {

    println!("Which label do you want to use for the build passed status?");

    let mut counter = 1;
    for i in 0..board_label_list.labels.len() {
        //Print with color
        println!("[{}] ({}) {}", i + 1, board_label_list.labels[i].color, board_label_list.labels[i].name);
        counter += 1;
    }
    writeln_green!(term, "[{}] Create a new label.", counter);
    writeln_red!(term, "[0] Quit.");

    let mut option: usize = 0;
    loop {

        get_input_usize!(term, &mut option, "Please enter an option: ");

        if option <= counter && option >= 0 {
            break;
        }else {
            writeln_red!(term, "Please enter a valid option.");
        }
    }

    if option == counter {
        match create_label_pass(term, &config, board_info){//
            Ok(_)    => (),//
            Err(err) => {
                panic!(format!("An error occured: {}", err));
            }
        }
    } else if option == 0 {
        exit(0)
    } else{
        board_info.build_pass_id = board_label_list.labels[option - 1].id.clone();//
    }

}

pub fn board_label_selection_fail(term: &mut Box<term::StdoutTerminal>, config: &mut config::TrelloBSTAPIConfig, board_label_list: &mut BoardsLabelsResponse, board_info: &mut TrelloBoardInfo) {

    println!("Which label do you want to use for the build failed status?");

    let mut counter = 1;
    for i in 0..board_label_list.labels.len() {
        //Print with color
        println!("[{}] ({}) {}", i + 1, board_label_list.labels[i].color, board_label_list.labels[i].name);
        counter += 1;
    }
    writeln_green!(term, "[{}] Create a new label.", counter);
    writeln_red!(term, "[0] Quit.");

    let mut option: usize = 0;
    loop {

        get_input_usize!(term, &mut option, "Please enter an option: ");

        if option <= counter && option >= 0 {
            break;
        }else {
            writeln_red!(term, "Please enter a valid option.");
        }
    }

    if option == counter {
        match create_label_fail(term, &config, board_info){//
            Ok(_)    => (),
            Err(err) => {
                panic!(format!("An error occured: {}", err));
            }
        }
    } else if option == 0 {
        exit(0)
    } else{
        board_info.build_fail_id = board_label_list.labels[option - 1].id.clone();//
    }

}

//NOTE: A label with no color is currently not supported, Serde-json feakes out when expecting a string and receiving a null
//TODO?: Create a manual parser in acquire_board_label_list.
pub fn create_label_pass(term: &mut Box<term::StdoutTerminal>, config: &config::TrelloBSTAPIConfig, board_info: &mut TrelloBoardInfo) -> Result<(), &'static str> {

    let mut label_name       = String::new();
    let mut label_color      = String::new();
    let     valid_colors     = ["green", "yellow", "orange", "red", "pink", "purple", "blue", "sky", "lime", "black"/*, "none"*/];
    let mut is_input_success = false;
    loop {
        get_input_string_success!(term, &mut label_name, &mut is_input_success, "Please enter a name for the label which will be in the build passed status: ");
        if is_input_success {break;}
    }

    loop {
        get_input_string_success!(term, &mut label_color, &mut is_input_success, "Please enter the color for the label which will be in the build passed status (Options are: Green, Yellow, Orange, Red, Pink, Purple, Blue, Sky, Lime, Black): ");
        if is_input_success {
            label_color = label_color.to_lowercase();
            if valid_colors.contains(&&label_color[..]) {break;}
            writeln_red!(term, "Please enter a valid color.");
        }
    }

    //if label_color == "none" {label_color = "".to_string()}

    let status            = utils::StatusPrint::from_str(term, "Creating the label.");
    let api_call          = format!("https://trello.com/1/board/{}/labels?name={}&color={}&key={}&token={}", board_info.board_id, label_name, label_color, config::trello_api_key, config.trello_app_token);
    let mut response_body = String::new();

    match utils::rest_api_call_post(&api_call) {
        Ok(_response_body) => response_body = _response_body,
        Err(err)           => {
            status.error(term);
            return Err(err);
        }
    }

    match utils::get_single_json_value_as_string(&response_body, "id") {
        Ok(value) => board_info.build_pass_id = value,
        Err(err)  => {
            status.error(term);
            return Err(err);
        }
    }

    status.success(term);
    Ok(())
}

//NOTE: A label with no color is currently not supported, Serde-json feakes out when expecting a string and receiving a null
//TODO?: Create a manual parser in acquire_board_label_list.
pub fn create_label_fail(term: &mut Box<term::StdoutTerminal>, config: &config::TrelloBSTAPIConfig, board_info: &mut TrelloBoardInfo) -> Result<(), &'static str> {

    let mut label_name       = String::new();
    let mut label_color      = String::new();
    let     valid_colors     = ["green", "yellow", "orange", "red", "pink", "purple", "blue", "sky", "lime", "black"/*, "none"*/];
    let mut is_input_success = false;
    loop {
        get_input_string_success!(term, &mut label_name, &mut is_input_success, "Please enter a name for the label which will in be the build failed status: ");
        if is_input_success {break;}
    }

    loop {
        get_input_string_success!(term, &mut label_color, &mut is_input_success, "Please enter the color for the label which will be in the build failed status (Options are: Green, Yellow, Orange, Red, Pink, Purple, Blue, Sky, Lime, Black): ");
        if is_input_success {
            label_color = label_color.to_lowercase();
            if valid_colors.contains(&&label_color[..]) {break;}
            writeln_red!(term, "Please enter a valid color.");
        }
    }

    //if label_color == "none" {label_color = "\"\"".to_string()}

    let status            = utils::StatusPrint::from_str(term, "Creating the label.");
    let api_call          = format!("https://trello.com/1/board/{}/labels?name={}&color={}&key={}&token={}", board_info.board_id, label_name, label_color, config::trello_api_key, config.trello_app_token);
    let mut response_body = String::new();

    match utils::rest_api_call_post(&api_call) {
        Ok(_response_body) => response_body = _response_body,
        Err(err)           => {
            status.error(term);
            return Err(err);
        }
    }

    match utils::get_single_json_value_as_string(&response_body, "id") {
        Ok(value) => board_info.build_fail_id = value,
        Err(err)  => {
            status.error(term);
            return Err(err);
        }
    }

    status.success(term);
    Ok(())
}

























