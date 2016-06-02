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

use std::collections::BTreeMap;
use std::error::Error;
use std::io;
use std::process::exit;

use config;

extern crate term;

use serde_json::Value;


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

pub struct LabelInfo {
    pub id:    String,
    pub name:  String,
    pub color: String
}

pub struct BoardsLabelsResponse {
    pub id:     String,
    pub labels: Vec<LabelInfo>
}

pub struct Trello {
    is_board_created: bool
}

impl Trello {

    pub fn new() -> Trello {
        Trello {
            is_board_created: false
        }
    }

    pub fn setup_api_token(&mut self, term: &mut Box<term::StdoutTerminal>, trello_api_key: &str, config: &mut config::TrelloBSTConfig) {

        let trello_app_token_config_key = "trello_app_token";
        let trello_app_token            = config.get(&trello_app_token_config_key);

        //If its empty then either were not loading a config from file or its first time use.
        if trello_app_token.is_empty() {
            let mut app_token = String::new();
            println!("Setting up Trello API Token...");
            get_input_string!(term, &mut app_token, "Log in to Trello.com and enter the app token from https://trello.com/1/authorize?response_type=token&key={}&scope=read%2Cwrite&expiration=never&name=TrelloBST : ", trello_api_key);
            config.set(&trello_app_token_config_key, &app_token);
        }
    }

    pub fn setup_board(&mut self, term: &mut Box<term::StdoutTerminal>, trello_api_key: &str, config: &mut config::TrelloBSTConfig) -> Result<(), &'static str> {

        //Get list of boards
        let     status                      = utils::StatusPrint::from_str(term, "Acquiring board list from Trello.");
        let     trello_app_token_config_key = "trello_app_token";
        let mut board_list                  = MembersMeBoardsResponse::new();
        let mut response_body               = String::new();
        let mut api_call                    = format!("https://api.trello.com/1/members/me?fields=&boards=open&board_fields=name&key={}&token={}", trello_api_key, config.get(trello_app_token_config_key));

        //  Do API call
        match utils::rest_api_call_get(&api_call) {
            Ok(_response_body) => response_body = _response_body,
            Err(err)           => {
                status.error(term);
                return Err(err);
            }
        }

        //  Parse JSON data
        match serde_json::from_str(&response_body){
            Ok(_board_list) => board_list = _board_list,
            Err(_)          => {
                status.error(term);
                return Err("Error parsing the response.");
            }
        }
        status.success(term);


        //Select Board.
        let mut board_select: utils::MenuBuilder<u64> = utils::MenuBuilder::new("Which board do you want to setup?".to_string());
        let mut counter:      u64                     = 1;
        let mut create_board: bool                    = false;

        //  Create menu
        for i in 0..board_list.boards.len() {
            board_select.add_entry(board_list.boards[i].name.clone(), i as u64 + 1);
            counter += 1;
        }
        board_select.add_entry_color(term::color::GREEN, "Create a new board.".to_string(), counter);

        //  Select
        let board_selected = board_select.select(term);
        if *board_selected == counter {
            create_board = true;
        } else {
            let index = (*board_selected - 1) as usize;
            create_board = false;
            config.set("trello_board_id", &board_list.boards[index].id.clone()[..]);
        }


        //Create board if wanted.
        if create_board {

            //Get new board name
            let mut board_name                  = String::new();
            let mut is_input_success: bool      = false;
            loop {
                get_input_string_success!(term, &mut board_name, &mut is_input_success, "Please enter a name for the new board: ");
                if is_input_success {break;}
            }

            //Create board
            let     status        = utils::StatusPrint::from_str(term, "Creating new board.");
            let     api_call      = format!("https://trello.com/1/boards?name={}&defaultLists=false&key={}&token={}", board_name, trello_api_key, config.get(trello_app_token_config_key));
            let mut response_body = String::new();

            //  Do API call
            match utils::rest_api_call_post(&api_call) {
                Ok(_response_body) => response_body = _response_body,
                Err(err)           => {
                    status.error(term);
                    return Err(err);
                }
            }

            //Get ID of the new board
            match utils::get_single_json_value_as_string(&response_body, "id") {
                Ok(value) => {config.set("trello_board_id", &value[..]);},
                Err(err)  => {
                    status.error(term);
                    return Err(err)
                }
            }
            self.is_board_created = true;
            status.success(term);
        }
        Ok(())
    }

    pub fn setup_list(&mut self, term: &mut Box<term::StdoutTerminal>, trello_api_key: &str, config: &mut config::TrelloBSTConfig) -> Result<(), &'static str> {

        //Acquire board list and select a list if board wasnt just created
        let     trello_app_token_config_key = "trello_app_token";
        let mut create_list                 = false;
        if !self.is_board_created {

            //Acquire board list
            let     status           = utils::StatusPrint::from_str(term, "Acquiring board's lists list from Trello.");
            let     api_call         = format!("https://api.trello.com/1/boards/{}?lists=open&list_fields=name&fields=name,desc&key={}&token={}", config.get("trello_board_id"), trello_api_key, config.get(trello_app_token_config_key));
            let mut response_body    = String::new();
            let mut board_lists_list = BoardsResponse::new();

            match utils::rest_api_call_get(&api_call) {
                Ok(_response_body) => response_body = _response_body,
                Err(err)           => return Err(err)
            }

            match serde_json::from_str(&response_body){
                Ok(_board_lists_list) => board_lists_list = _board_lists_list,
                Err(_)                => return Err("Error parsing the response.",)
            }

            //Select board list
            let mut board_list_select: utils::MenuBuilder<u64> = utils::MenuBuilder::new("Which board list do you want to use for the build statuses?".to_string());
            let mut counter:           u64                     = 1;

            for i in 0..board_lists_list.lists.len() {
                board_list_select.add_entry(board_lists_list.lists[i].name.clone(), i as u64 + 1);
                counter += 1;
            }

            board_list_select.add_entry_color(term::color::GREEN, "Create a new list.".to_string(), counter);

            let board_list_selected = board_list_select.select(term);
            if *board_list_selected == counter {
                create_list = true;
            } else {
                let index = (*board_list_selected - 1) as usize;
                config.set("trello_list_id", &board_lists_list.lists[index].id.clone()[..]);
            }
        } else {
            create_list = true;
        }


        //create board if wanted
        if create_list {
            let mut list_name        = String::new();
            let mut is_input_success = false;
            loop {
                get_input_string_success!(term, &mut list_name, &mut is_input_success, "Please enter a name for the list which will contain the build statuses: ");
                if is_input_success {break;}
            }

            let     trello_app_token_config_key = "trello_app_token";
            let     api_call                    = format!("https://trello.com/1/lists?name={}&idBoard={}&defaultLists=false&key={}&token={}", list_name, config.get("trello_board_id"), trello_api_key, config.get(trello_app_token_config_key));
            let mut response_body               = String::new();
            let     status                      = utils::StatusPrint::from_str(term, "Creating the list.");

            match utils::rest_api_call_post(&api_call) {
                Ok(_response_body) => response_body = _response_body,
                Err(err)           => {
                    status.error(term);
                    return Err(err);
                }
            }

            match utils::get_single_json_value_as_string(&response_body, "id") {
                Ok(value) => config.set("trello_list_id", &value[..]),
                Err(err)  => {
                    status.error(term);
                    return Err(err);
                }
            }
            status.success(term);
        }

        Ok(())
    }


    pub fn setup_labels(&mut self, term: &mut Box<term::StdoutTerminal>, trello_api_key: &str, config: &mut config::TrelloBSTConfig) -> Result<(), &'static str> {


        //Acquire board labels and select ones to be used if board was not just created.
        let     trello_app_token_config_key = "trello_app_token";
        if !self.is_board_created {

            //Acquire label list
            let     api_call         = format!("https://api.trello.com/1/boards/{}?labels=all&label_fields=name,color&fields=none&key={}&token={}", config.get("trello_board_id"), trello_api_key, config.get(trello_app_token_config_key));
            let mut response_body    = String::new();
            let mut board_label_list = BoardsLabelsResponse::new();
            let     status           = utils::StatusPrint::from_str(term, "Acquiring board's labels from Trello.");

            match utils::rest_api_call_get(&api_call) {
                Ok(_response_body) => response_body = _response_body,
                Err(err)           => return Err(err)
            }

            let board_labels: BoardsLabelsResponse;
            match BoardsLabelsResponse::from_json(&response_body) {
                Ok(_local_board_labels_response) => board_labels = _local_board_labels_response,
                Err(err)                         => return Err(err)
            }

            //Select labels
            let mut label_select: utils::MenuBuilder<u64> = utils::MenuBuilder::new(String::new());
            let mut counter:      u64                     = 1;

            for i in 0..board_labels.labels.len() {
                label_select.add_entry(format!(" ({}) {}", board_labels.labels[i].color.clone().to_uppercase(), board_labels.labels[i].name.clone()), i as u64 + 1);
                counter += 1;
            }

            label_select.add_entry_color(term::color::GREEN, "Create a new board.".to_string(), counter);

            //  Select pass label and create if returned true
            if Trello::select_label(term, config, &board_labels, &mut label_select, "pass", &counter) {
                //create
                match Trello::create_label(term, config, &trello_api_key, "pass") {
                    Ok(()) => (),
                    Err(err) => return Err(err)
                }
            }

            //  Select fail label and create if returned true
            if Trello::select_label(term, config, &board_labels, &mut label_select, "fail", &counter) {
                //create
                match Trello::create_label(term, config, &trello_api_key, "fail") {
                    Ok(()) => (),
                    Err(err) => return Err(err)
                }
            }

        } else {

            //Create pass label
            match Trello::create_label(term, config, &trello_api_key, "pass") {
                Ok(()) => (),
                Err(err) => return Err(err)
            }

            //Create fail label
            match Trello::create_label(term, config, &trello_api_key, "fail") {
                Ok(()) => (),
                Err(err) => return Err(err)
            }
        }
        Ok(())
    }

    pub fn select_label(term: &mut Box<term::StdoutTerminal>, config: &mut config::TrelloBSTConfig, board_labels: &BoardsLabelsResponse, menu_builder: &mut utils::MenuBuilder<u64>, pass_fail_txt: &str, counter: &u64) -> bool {

        println!("\nWhich label do you want to use for the build {} status?", &pass_fail_txt);

        //Select label
        let     label_selected = menu_builder.select(term);
        let mut create_label   = false;
        if *label_selected == *counter {
            create_label = true;
        } else {
            let index = (*label_selected - 1) as usize;
            config.set(&format!("trello_label_{}_id", &pass_fail_txt)[..], &board_labels.labels[index].id.clone()[..]);
        }
        return create_label;
    }

    pub fn create_label(term: &mut Box<term::StdoutTerminal>, config: &mut config::TrelloBSTConfig, trello_api_key: &str, pass_fail_txt: &str) -> Result<(), &'static str> {

        let mut label_name       = String::new();
        let mut label_color      = String::new();
        let     valid_colors     = ["green", "yellow", "orange", "red", "pink", "purple", "blue", "sky", "lime", "black", "none"];
        let mut is_input_success = false;

        //Get label name and color
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

        //Create label.
        let mut response_body               = String::new();
        let     trello_app_token_config_key = "trello_app_token";
        let     api_call                    = format!("https://trello.com/1/board/{}/labels?name={}&color={}&key={}&token={}", config.get("trello_board_id"), label_name, label_color, trello_api_key, config.get(trello_app_token_config_key));
        let     status                      = utils::StatusPrint::from_str(term, "Creating the label.");

        match utils::rest_api_call_post(&api_call) {
            Ok(_response_body) => response_body = _response_body,
            Err(err)           => {
                status.error(term);
                return Err(err);
            }
        }

        match utils::get_single_json_value_as_string(&response_body, "id") {
            Ok(value) => {config.set(&format!("trello_build_{}_id", &pass_fail_txt)[..], &value[..]);},
            Err(err)  => {
                status.error(term);
                return Err(err);
            }
        }

        status.success(term);
        Ok(())
    }
}

////////////////////////////////////////////////////////////
//                         Impls                          //
////////////////////////////////////////////////////////////

//NOTE: values to be moved to config sys
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

    pub fn from_json(json_data: &String) -> Result<BoardsLabelsResponse, &'static str> {

        let mut tmp_boards_labels_response = BoardsLabelsResponse::new();

        //Parse
        let data: Value;
        match serde_json::from_str(&json_data){
            Ok(_data) => data = _data,
            Err(_)    => return Err("Error parsing the JSON data")
        }

        //Get JSON object
        let object: BTreeMap<String, Value>;
        match data.as_object().ok_or("Error: JSON data does not describe an object.") {
            Ok(_object) => object = _object.clone(),
            Err(err)    => return Err(err)
        }

        //Get "id" field
        let id_value: Value;
        match object.get("id").ok_or("Error: The \"id\" field has not been found in the JSON data.") {
            Ok(_id_value) => id_value = _id_value.clone(),
            Err(err)      => return Err(err)
        }

        //Get "id" value.
        match id_value.as_string().ok_or("Error: The \"id\" field does not describe a string value.") {
            Ok(_response_id) => tmp_boards_labels_response.id = _response_id.to_string().clone(),
            Err(err)         => return Err(err)
        }

        //Get "labels" field
        let labels_value: Value;
        match object.get("labels").ok_or("Error: The \"labels\" field has not been found in the JSON data.") {
            Ok(_labels_value) => labels_value = _labels_value.clone(),
            Err(err)          => return Err(err)
        }

        //Get "labels" content
        let labels_array: Vec<Value>;
        match labels_value.as_array().ok_or("Error: The \"labels\" field does not describe an array.") {
            Ok(_labels_array) => labels_array = _labels_array.clone(),
            Err(err)          => return Err(err)
        }

        //For each label
        for label in &labels_array {

            //Get label object
            let mut label_object: BTreeMap<String, Value>;
            label_object = BTreeMap::new();
            match label.as_object().ok_or("Error: An entry in the \"labels\" field does not describe an object.") {
                Ok(_label_object) => label_object = _label_object.clone(),
                Err(err)          => return Err(err)
            }

            //Get "id" field
            let label_id_value: Value;
            match label_object.get("id").ok_or("Error: Failed to acquire the \"id\" field in a \"labels\" field.") {
                Ok(_label_id_value) => label_id_value = _label_id_value.clone(),
                Err(err)          => return Err(err)
            }

            let label_id_string: String;
            match label_id_value.as_string().ok_or("Error: Failed to convert the \"id\" field into a string.") {
                Ok(_label_id_string) => label_id_string = _label_id_string.to_string().clone(),
                Err(err)             => return Err(err)
            }

            //Get "name" field
            let label_name_value: Value;
            match label_object.get("name").ok_or("Error: Failed to acquire the \"name\" field in a \"labels\" field.") {
                Ok(_label_name_value) => label_name_value = _label_name_value.clone(),
                Err(err)              => return Err(err)
            }

            let label_name_string: String;
            match label_name_value.as_string().ok_or("Error: Failed to convert the value of the \"name\" field to a string.") {
                Ok(_label_name_string) => label_name_string = _label_name_string.to_string().clone(),
                Err(err)               => return Err(err)
            }

            //Get "color" field, if null, color is none
            let label_color_value: Value;
            match label_object.get("color").ok_or("Error: Failed to acquire the \"color\" field in a \"labels\" field.") {
                Ok(_label_color_value) => label_color_value = _label_color_value.clone(),
                Err(err)               => return Err(err)
            }

            let label_color_string: String;
            if label_color_value.is_null() {
                label_color_string = "none".to_string();
            } else {
                match label_color_value.as_string().ok_or("Error: Failed to convert the value of the \"color\" field to a string.") {
                    Ok(_label_color_string) => label_color_string = _label_color_string.to_string().clone(),
                    Err(err)                => return Err(err)
                }
            }

            tmp_boards_labels_response.labels.push(LabelInfo{
                id:    label_id_string,
                name:  label_name_string,
                color: label_color_string
            });
        }
        Ok(tmp_boards_labels_response)
    }
}


////////////////////////////////////////////////////////////
//                       Functions                        //
////////////////////////////////////////////////////////////


//pub fn setup_api_token(term: &mut Box<term::StdoutTerminal>, trello_api_key: &str, config: &mut config::TrelloBSTConfig) {
//
//    let trello_app_token_config_key = "trello_app_token";
//    let trello_app_token            = config.get(&trello_app_token_config_key);
//
//    //If its empty then either were not loading a config from file or its first time use.
//    if trello_app_token.is_empty() {
//        let mut app_token = String::new();
//        println!("Setting up Trello API Token...");
//        get_input_string!(term, &mut app_token, "Log in to Trello.com and enter the app token from https://trello.com/1/authorize?response_type=token&key={}&scope=read%2Cwrite&expiration=never&name=TrelloBST : ", trello_api_key);
//        config.set(&trello_app_token_config_key, &app_token);
//    }
//}
//
//
//pub fn setup_board(term: &mut Box<term::StdoutTerminal>, trello_api_key: &str, config: &mut config::TrelloBSTConfig, board_info: &mut TrelloBoardInfo) -> bool{
//
//    //Aquire board list
//    let mut board_list = MembersMeBoardsResponse::new();
//    let     status     = utils::StatusPrint::from_str(term, "Acquiring board list from Trello.");
//    match acquire_board_list(&trello_api_key, config, &mut board_list) {
//        Ok(_)    => {
//            status.success(term);
//        },
//        Err(err) => {
//            status.error(term);
//            panic!(format!("An error occurred while communicating with Trello: {}", err));
//        },
//    }
//
//    //Board selection.
//    board_selection(term, &trello_api_key, config, &mut board_list, board_info)
//}

//#[allow(unused_assignments)]
//pub fn acquire_board_list(trello_api_key: &str, config: &mut config::TrelloBSTConfig, board_list: &mut MembersMeBoardsResponse) -> Result<(), &'static str>{
//
//    let     trello_app_token_config_key = "trello_app_token";
//    let     api_call                    = format!("https://api.trello.com/1/members/me?fields=&boards=open&board_fields=name&key={}&token={}", trello_api_key, config.get(trello_app_token_config_key));
//    let mut response_body               = String::new();
//
//    match utils::rest_api_call_get(&api_call) {
//        Ok(_response_body) => response_body = _response_body,
//        Err(err)           => return Err(err)
//    }
//
//    let local_board_list: MembersMeBoardsResponse;
//    match serde_json::from_str(&response_body){
//        Ok(_board_list) => local_board_list = _board_list,
//        Err(_)          => return Err("Error parsing the response.")
//    }
//
//    board_list.id     = local_board_list.id;
//    board_list.boards = local_board_list.boards;
//
//    Ok(())
//}

//pub fn board_selection(term: &mut Box<term::StdoutTerminal>, trello_api_key: &str, config: &mut config::TrelloBSTConfig, board_list: &mut MembersMeBoardsResponse, board_info: &mut TrelloBoardInfo) -> bool {
//
//    let mut board_select: utils::MenuBuilder<u64> = utils::MenuBuilder::new("Which board do you want to setup?".to_string());
//    let mut counter:      u64                     = 1;
//    for i in 0..board_list.boards.len() {
//        board_select.add_entry(board_list.boards[i].name.clone(), i as u64 + 1);
//        counter += 1;
//    }
//
//    board_select.add_entry_color(term::color::GREEN, "Create a new board.".to_string(), counter);
//
//    let mut is_board_created = false;
//    let     board_selected   = board_select.select(term);
//    if *board_selected == counter {
//        match create_board(term, &trello_api_key, config, board_info){
//            Ok(_)    => is_board_created = true,
//            Err(err) => {
//                panic!(format!("An error occured: {}", err));
//            }
//        }
//    } else {
//        let index = (*board_selected - 1) as usize;
//        board_info.board_id = board_list.boards[index].id.clone();
//    }
//
//    return is_board_created;
//}

//#[allow(unused_assignments)]
//pub fn create_board(term: &mut Box<term::StdoutTerminal>, trello_api_key: &str, config: &mut config::TrelloBSTConfig, board_info: &mut TrelloBoardInfo) -> Result<(), &'static str> {
//
//    //Create board
//    let     trello_app_token_config_key = "trello_app_token";
//    let mut board_name                  = String::new();
//    let mut is_input_success: bool      = false;
//    loop {
//        get_input_string_success!(term, &mut board_name, &mut is_input_success, "Please enter a name for the new board: ");
//        if is_input_success {break;}
//    }
//
//    let     status = utils::StatusPrint::from_str(term, "Creating the board.");
//    let     api_call      = format!("https://trello.com/1/boards?name={}&defaultLists=false&key={}&token={}", board_name, trello_api_key, config.get(trello_app_token_config_key));
//    let mut response_body = String::new();
//
//    match utils::rest_api_call_post(&api_call) {
//        Ok(_response_body) => response_body = _response_body,
//        Err(err)           => {
//            status.error(term);
//            return Err(err);
//        }
//    }
//
//    match utils::get_single_json_value_as_string(&response_body, "id") {
//        Ok(value) => board_info.board_id = value,
//        Err(err)  => {
//            status.error(term);
//            return Err(err)
//        }
//    }
//
//    status.success(term);
//    Ok(())
//}
//pub fn setup_list(term: &mut Box<term::StdoutTerminal>, trello_api_key: &str, config: &mut config::TrelloBSTConfig, board_info: &mut TrelloBoardInfo, is_board_created: &bool) {
//
//    if !*is_board_created {
//
//        //If the board wasn't just created, acquire list
//        let     status           = utils::StatusPrint::from_str(term, "Acquiring board's lists list from Trello.");
//        let mut board_lists_list = BoardsResponse::new();
//        match acquire_board_lists_list(trello_api_key, config, &board_info, &mut board_lists_list) {
//            Ok(_)    => {
//                status.success(term);
//            },
//            Err(err) => {
//                status.error(term);
//                panic!(format!("An error occurred while communicating with Trello: {}", err));
//            },
//        }
//
//        //And list selection
//
//        board_list_selection(term, trello_api_key, config, &mut board_lists_list, board_info);
//
//    } else {
//        //If the board was just created then create the list also.
//        match create_list(term, trello_api_key, config, board_info){
//            Ok(_)    => (),
//            Err(err) => {
//                panic!(format!("An error occured: {}", err));
//            }
//        }
//    }
//}
//
//
//
//#[allow(unused_assignments)]
//pub fn acquire_board_lists_list(trello_api_key: &str, config: &mut config::TrelloBSTConfig, board_info: &TrelloBoardInfo, board_lists_list: &mut BoardsResponse) -> Result<(), &'static str>{
//
//    let     trello_app_token_config_key = "trello_app_token";
//    let     api_call                    = format!("https://api.trello.com/1/boards/{}?lists=open&list_fields=name&fields=name,desc&key={}&token={}", board_info.board_id, trello_api_key, config.get(trello_app_token_config_key));
//    let mut response_body               = String::new();
//
//    match utils::rest_api_call_get(&api_call) {
//        Ok(_response_body) => response_body = _response_body,
//        Err(err)           => return Err(err)
//    }
//
//    let tmp_board_lists_list: BoardsResponse;
//    match serde_json::from_str(&response_body){
//        Ok(_board_lists_list) => tmp_board_lists_list = _board_lists_list,
//        Err(_)                => return Err("Error parsing the response.",)
//    }
//
//    board_lists_list.id    = tmp_board_lists_list.id;
//    board_lists_list.name  = tmp_board_lists_list.name;
//    board_lists_list.desc  = tmp_board_lists_list.desc;
//    board_lists_list.lists = tmp_board_lists_list.lists;
//
//    Ok(())
//
//}

//pub fn board_list_selection(term: &mut Box<term::StdoutTerminal>, trello_api_key: &str, config: &mut config::TrelloBSTConfig, board_lists_list: &mut BoardsResponse, board_info: &mut TrelloBoardInfo) {
//
//    let mut board_list_select: utils::MenuBuilder<u64> = utils::MenuBuilder::new("Which board list do you want to use for the build statuses?".to_string());
//    let mut counter:           u64                     = 1;
//
//    for i in 0..board_lists_list.lists.len() {
//        board_list_select.add_entry(board_lists_list.lists[i].name.clone(), i as u64 + 1);
//        counter += 1;
//    }
//
//    board_list_select.add_entry_color(term::color::GREEN, "[{}] Create a new list.".to_string(), counter);
//
//    let board_list_selected = board_list_select.select(term);
//    if *board_list_selected == counter {
//        /////////////////////////////////////////////////////////////////////////////////////////////
//        match create_list(term, trello_api_key, config, board_info){
//            Ok(_)    => (),
//            Err(err) => {
//                panic!(format!("An error occured: {}", err));
//            }
//        }
//    } else {
//        let index = (*board_list_selected - 1) as usize;
//        board_info.list_id = board_lists_list.lists[index].id.clone();
//    }
//}

//#[allow(unused_assignments)]
//pub fn create_list(term: &mut Box<term::StdoutTerminal>, trello_api_key: &str, config: &mut config::TrelloBSTConfig, board_info: &mut TrelloBoardInfo) -> Result<(), &'static str> {
//
//    let mut list_name        = String::new();
//    let mut is_input_success = false;
//    loop {
//        get_input_string_success!(term, &mut list_name, &mut is_input_success, "Please enter a name for the list which will contain the build statuses: ");
//        if is_input_success {break;}
//    }
//
//    let     status                      = utils::StatusPrint::from_str(term, "Creating the list.");
//    let     trello_app_token_config_key = "trello_app_token";
//    let     api_call                    = format!("https://trello.com/1/lists?name={}&idBoard={}&defaultLists=false&key={}&token={}", list_name, board_info.board_id, trello_api_key, config.get(trello_app_token_config_key));
//    let mut response_body               = String::new();
//
//    match utils::rest_api_call_post(&api_call) {
//        Ok(_response_body) => response_body = _response_body,
//        Err(err)           => {
//            status.error(term);
//            return Err(err);
//        }
//    }
//
//    match utils::get_single_json_value_as_string(&response_body, "id") {
//        Ok(value) => board_info.list_id = value,
//        Err(err)  => {
//            status.error(term);
//            return Err(err);
//        }
//    }
//
//    status.success(term);
//    Ok(())
//}


//pub fn setup_labels(term: &mut Box<term::StdoutTerminal>, trello_api_key: &str, config: &mut config::TrelloBSTConfig, board_info: &mut TrelloBoardInfo, is_board_created: &bool) {
//
//    if !*is_board_created {
//
//        //If the board wasn't just created, acquire list
//        let     status           = utils::StatusPrint::from_str(term, "Acquiring board's labels from Trello.");
//        let mut board_label_list = BoardsLabelsResponse::new();
//        ////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
//        match acquire_board_label_list(&config, &board_info, &mut board_label_list) {
//            Ok(_)    => {
//                status.success(term);
//            },
//            Err(err) => {
//                status.error(term);
//                panic!(format!("An error occurred while communicating with Trello: {}", err));
//            },
//        }
//
//        //And label selection
//        //Build pass
//        ////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
//        board_label_selection_pass(term, config, &mut board_label_list, board_info);
//        //Build fail
//        ////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
//        board_label_selection_fail(term, config, &mut board_label_list, board_info);
//
//    } else {
//        //If the board was just created then create the list also.
//        //////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
//        match create_label_pass(term, &config, board_info){//
//            Ok(_)    => (),//
//            Err(err) => {
//                panic!(format!("An error occured: {}", err));
//            }
//        }
//
//        ///////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
//        match create_label_fail(term, &config, board_info){//
//            Ok(_)    => (),//
//            Err(err) => {
//                panic!(format!("An error occured: {}", err));
//            }
//        }
//    }
//
//}

////TODO?: Create a manual parser.
//pub fn acquire_board_label_list(trello_api_key: &str, config: &config::TrelloBSTConfig, board_info: &TrelloBoardInfo, board_label_list: &mut BoardsLabelsResponse) -> Result<(), &'static str> {
//
//    let     trello_app_token_config_key = "trello_app_token";
//    let     api_call                    = format!("https://api.trello.com/1/boards/{}?labels=all&label_fields=name,color&fields=none&key={}&token={}",board_info.board_id, trello_api_key, config.get(trello_app_token_config_key));
//    let mut response_body               = String::new();
//
//    match utils::rest_api_call_get(&api_call) {
//        Ok(_response_body) => response_body = _response_body,
//        Err(err)           => return Err(err)
//    }
//
//    let local_board_lebels_response: BoardsLabelsResponse;
//    match serde_json::from_str(&response_body){
//        Ok(_board_lists_list) => local_board_lebels_response = _board_lists_list,
//        Err(_)                => return Err("Error parsing the response.",)
//    }
//
//    board_label_list.id     = local_board_lebels_response.id;
//    board_label_list.labels = local_board_lebels_response.labels;
//
//    Ok(())
//}
//
//pub fn board_label_selection_pass(term: &mut Box<term::StdoutTerminal>, config: &mut config::TrelloBSTIConfig, board_label_list: &mut BoardsLabelsResponse, board_info: &mut TrelloBoardInfo) {
//
//    println!("Which label do you want to use for the build passed status?");
//
//    let mut counter = 1;
//    for i in 0..board_label_list.labels.len() {
//        //Print with color
//        println!("[{}] ({}) {}", i + 1, board_label_list.labels[i].color, board_label_list.labels[i].name);
//        counter += 1;
//    }
//    writeln_green!(term, "[{}] Create a new label.", counter);
//    writeln_red!(term, "[0] Quit.");
//
//    let mut option: usize = 0;
//    loop {
//
//        get_input_usize!(term, &mut option, "Please enter an option: ");
//
//        if option <= counter {
//            break;
//        }else {
//            writeln_red!(term, "Please enter a valid option.");
//        }
//    }
//
//    if option == counter {
//        match create_label_pass(term, &config, board_info){//
//            Ok(_)    => (),//
//            Err(err) => {
//                panic!(format!("An error occured: {}", err));
//            }
//        }
//    } else if option == 0 {
//        exit(0)
//    } else{
//        board_info.build_pass_id = board_label_list.labels[option - 1].id.clone();//
//    }
//
//}
//
//pub fn board_label_selection_fail(term: &mut Box<term::StdoutTerminal>, config: &mut config::TrelloBSTConfig, board_label_list: &mut BoardsLabelsResponse, board_info: &mut TrelloBoardInfo) {
//
//    println!("Which label do you want to use for the build failed status?");
//
//    let mut counter = 1;
//    for i in 0..board_label_list.labels.len() {
//        //Print with color
//        println!("[{}] ({}) {}", i + 1, board_label_list.labels[i].color, board_label_list.labels[i].name);
//        counter += 1;
//    }
//    writeln_green!(term, "[{}] Create a new label.", counter);
//    writeln_red!(term, "[0] Quit.");
//
//    let mut option: usize = 0;
//    loop {
//
//        get_input_usize!(term, &mut option, "Please enter an option: ");
//
//        if option <= counter {
//            break;
//        }else {
//            writeln_red!(term, "Please enter a valid option.");
//        }
//    }
//
//    if option == counter {
//        match create_label_fail(term, &config, board_info){//
//            Ok(_)    => (),
//            Err(err) => {
//                panic!(format!("An error occured: {}", err));
//            }
//        }
//    } else if option == 0 {
//        exit(0)
//    } else{
//        board_info.build_fail_id = board_label_list.labels[option - 1].id.clone();//
//    }
//
//}

//NOTE: A label with no color is currently not supported, Serde-json feakes out when expecting a string and receiving a null
//TODO?: Create a manual parser in acquire_board_label_list.
//pub fn create_label_pass(term: &mut Box<term::StdoutTerminal>, config: &config::TrelloBSTConfig, board_info: &mut TrelloBoardInfo) -> Result<(), &'static str> {
//
//    let mut label_name       = String::new();
//    let mut label_color      = String::new();
//    let     valid_colors     = ["green", "yellow", "orange", "red", "pink", "purple", "blue", "sky", "lime", "black"/*, "none"*/];
//    let mut is_input_success = false;
//    loop {
//        get_input_string_success!(term, &mut label_name, &mut is_input_success, "Please enter a name for the label which will be in the build passed status: ");
//        if is_input_success {break;}
//    }
//
//    loop {
//        get_input_string_success!(term, &mut label_color, &mut is_input_success, "Please enter the color for the label which will be in the build passed status (Options are: Green, Yellow, Orange, Red, Pink, Purple, Blue, Sky, Lime, Black): ");
//        if is_input_success {
//            label_color = label_color.to_lowercase();
//            if valid_colors.contains(&&label_color[..]) {break;}
//            writeln_red!(term, "Please enter a valid color.");
//        }
//    }
//
//    //if label_color == "none" {label_color = "".to_string()}
//
//    let status            = utils::StatusPrint::from_str(term, "Creating the label.");
//    let api_call          = format!("https://trello.com/1/board/{}/labels?name={}&color={}&key={}&token={}", board_info.board_id, label_name, label_color, config::TRELLO_API_KEY, config.trello_app_token);
//    let mut response_body = String::new();
//
//    match utils::rest_api_call_post(&api_call) {
//        Ok(_response_body) => response_body = _response_body,
//        Err(err)           => {
//            status.error(term);
//            return Err(err);
//        }
//    }
//
//    match utils::get_single_json_value_as_string(&response_body, "id") {
//        Ok(value) => board_info.build_pass_id = value,
//        Err(err)  => {
//            status.error(term);
//            return Err(err);
//        }
//    }
//
//    status.success(term);
//    Ok(())
//}
//
////NOTE: A label with no color is currently not supported, Serde-json feakes out when expecting a string and receiving a null
////TODO?: Create a manual parser in acquire_board_label_list.
//pub fn create_label_fail(term: &mut Box<term::StdoutTerminal>, config: &config::TrelloBSTConfig, board_info: &mut TrelloBoardInfo) -> Result<(), &'static str> {
//
//    let mut label_name       = String::new();
//    let mut label_color      = String::new();
//    let     valid_colors     = ["green", "yellow", "orange", "red", "pink", "purple", "blue", "sky", "lime", "black"/*, "none"*/];
//    let mut is_input_success = false;
//    loop {
//        get_input_string_success!(term, &mut label_name, &mut is_input_success, "Please enter a name for the label which will in be the build failed status: ");
//        if is_input_success {break;}
//    }
//
//    loop {
//        get_input_string_success!(term, &mut label_color, &mut is_input_success, "Please enter the color for the label which will be in the build failed status (Options are: Green, Yellow, Orange, Red, Pink, Purple, Blue, Sky, Lime, Black): ");
//        if is_input_success {
//            label_color = label_color.to_lowercase();
//            if valid_colors.contains(&&label_color[..]) {break;}
//            writeln_red!(term, "Please enter a valid color.");
//        }
//    }
//
//    //if label_color == "none" {label_color = "\"\"".to_string()}
//
//    let status            = utils::StatusPrint::from_str(term, "Creating the label.");
//    let api_call          = format!("https://trello.com/1/board/{}/labels?name={}&color={}&key={}&token={}", board_info.board_id, label_name, label_color, config::TRELLO_API_KEY, config.trello_app_token);
//    let mut response_body = String::new();
//
//    match utils::rest_api_call_post(&api_call) {
//        Ok(_response_body) => response_body = _response_body,
//        Err(err)           => {
//            status.error(term);
//            return Err(err);
//        }
//    }
//
//    match utils::get_single_json_value_as_string(&response_body, "id") {
//        Ok(value) => board_info.build_fail_id = value,
//        Err(err)  => {
//            status.error(term);
//            return Err(err);
//        }
//    }
//
//    status.success(term);
//    Ok(())
//}

























