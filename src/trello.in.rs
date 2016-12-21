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

use std::io;

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

        let trello_api_token_config_key = "trello_api_token";
        let trello_api_token            = config.get(&trello_api_token_config_key);

        //If its empty then either were not loading a config from file or its first time use.
        if trello_api_token.is_empty() {
            let mut api_token = String::new();
            println!("Setting up Trello API Token...");
            get_input_string!(term, &mut api_token, "Log in to Trello.com and enter the app token from https://trello.com/1/authorize?response_type=token&key={}&scope=read%2Cwrite&expiration=never&name=TrelloBST : ", trello_api_key);
            config.set(&trello_api_token_config_key, &api_token);

            //Save config
            let status = utils::StatusPrint::from_str(term, "Saving configuration file...");
            match config.save() {
                Ok(())   => {status.success(term);},
                Err(err) => {
                    status.error(term);
                    writeln_red!(term, "Error: Failed to save the configuration file: {}, TrelloBST will continue without saving inputted values into the configuration file.", err);
                }
            }
        }
    }

    #[allow(unused_assignments)]
    pub fn setup_board(&mut self, term: &mut Box<term::StdoutTerminal>, trello_api_key: &str, config: &mut config::TrelloBSTConfig) -> Result<(), &'static str> {

        //Get list of boards
        let status                      = utils::StatusPrint::from_str(term, "Acquiring board list from Trello.");
        let trello_api_token_config_key = "trello_api_token";
        let api_call                    = format!("https://api.trello.com/1/members/me?fields=&boards=open&board_fields=name&key={}&token={}", trello_api_key, config.get(trello_api_token_config_key));

        //  Do API call
        let response_body = match utils::rest_api_call_get(&api_call) {
            Ok(response_body) => response_body,
            Err(err)          => {
                status.error(term);
                return Err(err);
            }
        };

        //  Parse JSON data
        let board_list: MembersMeBoardsResponse = match serde_json::from_str(&response_body){
            Ok(board_list) => board_list,
            Err(_)         => {
                status.error(term);
                return Err("Error parsing the response.");
            }
        };
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
            let     api_call      = format!("https://trello.com/1/boards?name={}&defaultLists=false&key={}&token={}", board_name, trello_api_key, config.get(trello_api_token_config_key));

            //  Do API call
            let response_body = match utils::rest_api_call_post(&api_call) {
                Ok(response) => response,
                Err(err)     => {
                    status.error(term);
                    return Err(err);
                }
            };

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
        let     trello_api_token_config_key = "trello_api_token";
        let mut create_list                 = false;
        if !self.is_board_created {

            //Acquire board list
            let     status           = utils::StatusPrint::from_str(term, "Acquiring board's lists list from Trello.");
            let     api_call         = format!("https://api.trello.com/1/boards/{}?lists=open&list_fields=name&fields=name,desc&key={}&token={}", config.get("trello_board_id"), trello_api_key, config.get(trello_api_token_config_key));

            let response_body = match utils::rest_api_call_get(&api_call) {
                Ok(response_body) => response_body,
                Err(err)           => {
                    status.error(term);
                    return Err(err)
                }
            };

            let board_lists_list: BoardsResponse = match serde_json::from_str(&response_body){
                Ok(board_lists_list) => board_lists_list,
                Err(_)               => {
                    status.error(term);
                    return Err("Error parsing the response.");
                }
            };

            status.success(term);

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

            let     trello_api_token_config_key = "trello_api_token";
            let     api_call                    = format!("https://trello.com/1/lists?name={}&idBoard={}&defaultLists=false&key={}&token={}", list_name, config.get("trello_board_id"), trello_api_key, config.get(trello_api_token_config_key));
            let     status                      = utils::StatusPrint::from_str(term, "Creating the list.");

            let response_body = match utils::rest_api_call_post(&api_call) {
                Ok(response_body) => response_body,
                Err(err)          => {
                    status.error(term);
                    return Err(err);
                }
            };

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
        let     trello_api_token_config_key = "trello_api_token";
        if !self.is_board_created {

            //Acquire label list
            let     api_call         = format!("https://api.trello.com/1/boards/{}?labels=all&label_fields=name,color&fields=none&key={}&token={}", config.get("trello_board_id"), trello_api_key, config.get(trello_api_token_config_key));
            let     status           = utils::StatusPrint::from_str(term, "Acquiring board's labels from Trello.");

            let response_body = match utils::rest_api_call_get(&api_call) {
                Ok(response_body) => response_body,
                Err(err)          => {
                    status.error(term);
                    return Err(err)
                }
            };

            let board_labels: BoardsLabelsResponse = match BoardsLabelsResponse::from_json(&response_body) {
                Ok(labels) => labels,
                Err(err)   => {
                    status.error(term);
                    return Err(err)
                }
            };

            status.success(term);

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
        let trello_api_token_config_key = "trello_api_token";
        let api_call                    = format!("https://trello.com/1/board/{}/labels?name={}&color={}&key={}&token={}", config.get("trello_board_id"), label_name, label_color, trello_api_key, config.get(trello_api_token_config_key));
        let status                      = utils::StatusPrint::from_str(term, "Creating the label.");

        let response_body = match utils::rest_api_call_post(&api_call) {
            Ok(response_body) => response_body,
            Err(err)          => {
                status.error(term);
                return Err(err);
            }
        };

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
        let data: Value = match serde_json::from_str(&json_data){
            Ok(data) => data,
            Err(_)   => return Err("Error parsing the JSON data")
        };

        //Get JSON object
        let object = try!(data.as_object().ok_or("Error: JSON data does not describe an object.")).clone();

        //Get "id" field
        let id_value: Value = try!(object.get("id").ok_or("Error: The \"id\" field has not been found in the JSON data.")).clone();

        //Get "id" value.
        tmp_boards_labels_response.id = try!(id_value.as_str().ok_or("Error: The \"id\" field does not describe a string value.")).to_string().clone();

        //Get "labels" field
        let labels_value: Value = try!(object.get("labels").ok_or("Error: The \"labels\" field has not been found in the JSON data.")).clone();

        //Get "labels" content
        let labels_array = try!(labels_value.as_array().ok_or("Error: The \"labels\" field does not describe an array.")).clone();

        //For each label
        for label in &labels_array {

            //Get label object
            let label_object = try!(label.as_object().ok_or("Error: An entry in the \"labels\" field does not describe an object.")).clone();

            //Get "id" field
            let label_id_value: Value = try!(label_object.get("id").ok_or("Error: Failed to acquire the \"id\" field in a \"labels\" field.")).clone();
            let label_id_string       = try!(label_id_value.as_str().ok_or("Error: Failed to convert the \"id\" field into a string.")).to_string().clone();

            //Get "name" field
            let label_name_value: Value = try!(label_object.get("name").ok_or("Error: Failed to acquire the \"name\" field in a \"labels\" field.")).clone();
            let label_name_string       = try!(label_name_value.as_str().ok_or("Error: Failed to convert the value of the \"name\" field to a string.")).to_string().clone();

            //Get "color" field, if null, color is none
            let label_color_value: Value = try!(label_object.get("color").ok_or("Error: Failed to acquire the \"color\" field in a \"labels\" field.")).clone();

            let label_color_string: String;
            if label_color_value.is_null() {
                label_color_string = "none".to_string();
            } else {
                label_color_string = try!(label_color_value.as_str().ok_or("Error: Failed to convert the value of the \"color\" field to a string.")).to_string().clone();
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
