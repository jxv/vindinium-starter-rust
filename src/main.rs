#![feature(globs)]

extern crate serialize;
extern crate http;
extern crate url;
use std::string::String;
use std::io::File;

use vindinium::*;
use bot::*;
mod vindinium;
mod bot;


// Main

fn main() {
    let settings = vindinium::Settings {
        key: get_key("key.txt"),
        url: "http://vindinium.org".to_string(),
        mode: Mode::Training(Some(100), Some("m1".to_string())),
    };
    let (url, obj) = start_msg(&settings);
    let mut state = match vindinium::request(url, obj) {
        Some(s) => s,
        None => { return (); }
    };
    let mut bot = RandomBot::new();
    loop {
        if state.game.turn >= state.game.heroes.len() as int {
            state.clear_pretty_print();
        }
        state.pretty_print();
        if state.game.finished {
            break;
        }
        bot = bot.step(&state);
        let (url, obj) = step_msg(&settings, &state, bot.dir());
        state = match request(url, obj) {
            Some(s) => s,
            None => { break; },
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
        Err(err) => panic!("{}", err),
    }
}

