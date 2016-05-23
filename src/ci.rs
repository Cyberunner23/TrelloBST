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
use std::cell::RefCell;

extern crate core;
use self::core::ops::DerefMut;

extern crate term;

use config;
use utils;


////////////////////////////////////////////////////////////
//                         Traits                         //
////////////////////////////////////////////////////////////

//NOTE: Trait objects derived from this trait must derive from clone (#[derive(Clone)])
pub trait CITrait {
    fn get_name(&mut self, ) -> String;
    fn setup(&mut self, term: &mut Box<term::StdoutTerminal>, config: &mut config::TrelloBSTConfig) -> Result<(), &'static str>;
    fn generate_ci_config(&mut self, config: &mut config::TrelloBSTConfig) -> Result<String, &'static str>;
}


////////////////////////////////////////////////////////////
//                        Structs                         //
////////////////////////////////////////////////////////////

pub struct CI {
    pub ci_map: BTreeMap<String, RefCell<Box<CITrait>>>
}

impl CI {

    pub fn new() -> CI {
        CI {
            ci_map: BTreeMap::new()
        }
    }

    pub fn register_ci(&mut self, mut ci: Box<CITrait>) {
        self.ci_map.insert(ci.get_name().to_string(), RefCell::new(ci));
    }

    pub fn generate_ci_config(&mut self, term: &mut Box<term::StdoutTerminal>, config: &mut config::TrelloBSTConfig) -> Result<String, &'static str> {

        //Select CI
        let mut ci_select = utils::MenuBuilder::new("Which Continuous Integration provider do you want a configuration for?".to_string());
        for (ci_name, ci_object) in self.ci_map.iter() {
            ci_select.add_entry(ci_name.clone(), ci_object);
        }

        let mut selected_ci = ci_select.select(term).borrow_mut();

        //Setup CI (get api keys and create repo menu + selection)
        match selected_ci.setup(term, config) {
            Ok(())   => (),
            Err(err) => return Err(err)
        }

        //Generate CI config
        match selected_ci.generate_ci_config(config) {
            Ok(ci_config_string) => return Ok(ci_config_string.clone()),
            Err(err)             => return Err(err)
        }
    }
}









