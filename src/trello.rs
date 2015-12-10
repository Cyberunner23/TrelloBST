#![cfg_attr(feature = "serde_macros", feature(custom_derive, plugin))]
#![cfg_attr(feature = "serde_macros", plugin(serde_macros))]

use std::env;

extern crate serde;
extern crate serde_json;

#[cfg(feature = "serde_macros")]
include!("trello.rs.in");

#[cfg(not(feature = "serde_macros"))]
include!(concat!(env::var("OUT_DIR"), "/trello.rs"));