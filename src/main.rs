#![feature(globs)]

extern crate serialize;
extern crate http;
extern crate url;
use std::string::String;
use std::io::File;
use self::serialize::json;

use vindinium::*;
use bot::*;
mod vindinium;
mod bot;


// Main

fn main() {
    let settings = vindinium::Settings {
        key: get_key("key.txt"),
        url: "http://vindinium.org".to_string(),
        mode: Training(Some(1), Some("m1".to_string())),
    };
    let (mut url, mut obj) = start(&settings);
    let mut state = match vindinium::request(url, obj) {
        Some(s) => s,
        None => { return (); }
    };
    let mut bot = RandomBot { dir: Stay };
    loop {
        if state.game.finished {
            break;
        }
        let dir = bot.move(&state);
        state = match step(&state, dir) {
            Some(s) => s,
            None => state,
        }
    }
}

fn get_key(filename: &str) -> String {
    let res_key = File::open(&Path::new(filename)).read_to_string();
    match res_key {
        Ok(key) => {
            let mut key_ = key.clone();
            key_.pop();
            key_
        }
        Err(err) => fail!("{}", err),
    }
}

