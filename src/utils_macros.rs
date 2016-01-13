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


////////////////////////////////////////////////////////////
//                         Macros                         //
////////////////////////////////////////////////////////////

macro_rules! match_to_none {
    ($match_expr:expr) => {
        match $match_expr {
            Ok(_)  => (),
            Err(_) => (),
        }
    }
}

macro_rules! writeln_red {
    ($term:expr, $($msg:tt)*) => {
        match_to_none!($term.fg(term::color::RED));
        match_to_none!($term.write_fmt(format_args!("{}\n", format_args!($($msg)*))));
        match_to_none!($term.flush());
        match_to_none!($term.reset());
    }
}

macro_rules! write_red {
    ($term:expr, $($msg:tt)*) => {
        match_to_none!($term.fg(term::color::RED));
        match_to_none!($term.write_fmt(format_args!("{}", format_args!($($msg)*))));
        match_to_none!($term.flush());
        match_to_none!($term.reset());
    }
}

macro_rules! writeln_green {
    ($term:expr, $($msg:tt)*) => {
        match_to_none!($term.fg(term::color::GREEN));
        match_to_none!($term.write_fmt(format_args!("{}\n", format_args!($($msg)*))));
        match_to_none!($term.flush());
        match_to_none!($term.reset());
    }
}

macro_rules! write_green {
    ($term:expr, $($msg:tt)*) => {
        match_to_none!($term.fg(term::color::GREEN));
        match_to_none!($term.write_fmt(format_args!("{}", format_args!($($msg)*))));
        match_to_none!($term.flush());
        match_to_none!($term.reset());
    }
}


macro_rules! get_input_usize {
    ($term:expr, $var:expr, $($msg:tt)*) => {{
        let mut input_str = String::new();
        match_to_none!($term.write_fmt(format_args!("{}", format_args!($($msg)*))));///////////////////////////
        match_to_none!($term.flush());
        match io::stdin().read_line(&mut input_str) {
            Ok(_)  => {
                input_str = input_str.trim_matches('\n').to_string();
                match input_str.parse::<usize>(){
                    Ok(input) => {
                        *$var = input;
                    },
                    Err(_)    => {
                        writeln_red!($term, "Error while parsing the input.");
                    }
                }
            },
            Err(_) => {panic!("Error while reading the input.");}
        }
    }};

    ($term:expr, $var:expr, $is_success:expr, $($msg:tt)*) => {{
        let mut input_str = String::new();
        match_to_none!($term.write_fmt(format_args!("{}", format_args!($($msg)*))));///////////////////////////////
        match_to_none!($term.flush());
        match io::stdin().read_line(&mut input_str) {
            Ok(_)  => {
                input_str = input_str.trim_matches('\n').to_string();
                match input_str.parse::<usize>(){
                    Ok(input) => {
                        *$var        = input;
                        *$is_success = true;
                    },
                    Err(_)    => {
                    *$is_success = false;
                        writeln_red!($term, "Error while parsing the input.");
                    }
                }
            },
            Err(_) => {panic!("Error while reading the input.");}
        }
    }}
}

macro_rules! get_input_usize_success {

($term:expr, $var:expr, $is_success:expr, $($msg:tt)*) => {{
    let mut input_str = String::new();
    match_to_none!($term.write_fmt(format_args!("{}", format_args!($($msg)*))));///////////////////////////////
    match_to_none!($term.flush());
        match io::stdin().read_line(&mut input_str) {
            Ok(_)  => {
                input_str = input_str.trim_matches('\n').to_string();
                match input_str.parse::<usize>(){
                    Ok(input) => {
                        *$var        = input;
                        *$is_success = true;
                    },
                    Err(_)    => {
                        *$is_success = false;
                        writeln_red!($term, "Error while parsing the input.");
                    }
                }
            },
            Err(_) => {panic!("Error while reading the input.");
        }
    }
    }}
}

macro_rules! get_input_string {
    ($term:expr, $var:expr, $($msg:tt)*) => {{
        let mut input_str = String::new();
        match_to_none!($term.write_fmt(format_args!("{}", format_args!($($msg)*))));////////////////////////////
        match_to_none!($term.flush());
        match io::stdin().read_line(&mut input_str) {
            Ok(_)  => {
                *$var = input_str.trim_matches('\n').to_string();
            },
            Err(_) => {panic!("Error while reading the input.");}
        }
    }};
}

macro_rules! get_input_string_success {
    ($term:expr, $var:expr, $is_success:expr, $($msg:tt)*) => {{
        let mut input_str = String::new();
        match_to_none!($term.write_fmt(format_args!("{}", format_args!($($msg)*))));///////////////////////////
        match_to_none!($term.flush());
        match io::stdin().read_line(&mut input_str) {
            Ok(_)  => {
                *$var = input_str.trim_matches('\n').to_string();
                *$is_success = true;
            },
            Err(_) => {panic!("Error while reading the input.");}
        }
    }}
}
