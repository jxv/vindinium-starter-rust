extern crate hyper;
extern crate url;
extern crate rustc_serialize;
use std::string::String;
use std::fs::File;
use std::path::Path;
use std::io::Read;
use self::rustc_serialize::json;

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
    let mut state = match vindinium::request(url, obj as json::Object) {
        Some(s) => s,
        None => { return (); }
    };
    let mut bot = RandomBot::new();
    loop {
        if state.game.turn >= state.game.heroes.len() as isize {
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
    let mut res_key = String::new();
    let res = File::open(&Path::new(filename)).unwrap().read_to_string(&mut res_key);
    match res {
        Ok(_) => {
            res_key.clone()
        }
        Err(err) => panic!("{}", err),
    }
}

