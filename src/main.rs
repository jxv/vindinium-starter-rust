#![feature(globs)]
extern crate serialize;
extern crate http;
extern crate url;
use std::os;
use std::collections::TreeMap;
use std::string::String;
use std::io::File;
use self::serialize::json;

use vindinium::*;
mod vindinium;


// Main

fn main() {
    let settings = vindinium::Settings {
        key: get_key(),
        url: "http://vindinium.org".to_string(),
        mode: Training(Some(1), Some("m1".to_string())),
    };
    let (url, obj) = start(&settings);
    let state = match vindinium::request(url, obj) {
        Some(s) => s,
        None => {
            return ()
        }
    };
    println!("{}", state.game.finished);
}

fn get_key() -> String {
    let filename = "key.txt";
    match read_key_from_file(filename) {
        Some(k) => k,
        None => fail!("can't read file \"{}\"", filename),
    }
}

fn read_key_from_file(filename: &str) -> Option<String> {
    let mut key = match File::open(&Path::new(filename)).read_to_string() {
        Ok(s) => s,
        Err(err) => {
            println!("{}", err);
            return None
        }
    };
    key.pop(); // drop the added newline char
    Some(key)
}

fn start(settings: &Settings) -> (String, json::JsonObject) {
    match settings.mode.clone() {
        Training(opt_turns, opt_map) => start_training(settings, opt_turns, opt_map),
        Arena => start_arena(settings),
    }
}

fn start_training(settings: &Settings, opt_turns: Option<u64>, opt_map: Option<String>) -> (String, json::JsonObject) {
    let mut obj: json::JsonObject = TreeMap::new();
    obj.insert("key".to_string(), json::String(settings.key.clone()));
    match opt_turns {
        Some(turns) => { obj.insert("turns".to_string(), json::U64(turns)); }, 
        None => (),
    };
    match opt_map {
        Some(map) => { obj.insert("map".to_string(), json::String(map)); },
        None => (),
    };
    (settings.start_url("training"), obj)
}

fn start_arena(settings: &Settings) -> (String, json::JsonObject) {
    let mut obj: json::JsonObject = TreeMap::new();
    obj.insert("key".to_string(), json::String(settings.key.clone()));
    
    (settings.start_url("arena"), obj)
}
