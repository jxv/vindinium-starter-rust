#![feature(globs)]
extern crate serialize;
extern crate http;
extern crate url;
use std::collections::TreeMap;
use std::string::String;
use std::io::File;
use self::serialize::json;
use http::client::RequestWriter;
use http::method::{Get,Post};
use url::Url;

use vindinium::*;
mod vindinium;



fn get_key() -> String {
    let mut key = match File::open(&Path::new("key.txt")).read_to_string() {
        Ok(s) => s,
        Err(err) => fail!("{}", err),
    };
    key.pop(); // drop the added newline char
    key
}

fn training(settings: &Settings, turns: u64, map: String) -> (String, json::JsonObject) {
    let url: String = settings.start_url("training");

    let mut obj: json::JsonObject = TreeMap::new();
    obj.insert("turns".to_string(), json::U64(turns));
    obj.insert("map".to_string(), json::String(map));
    
    (url, obj)
}


fn main() {
    let settings = vindinium::Settings {
        key: get_key(),
        url: "http://vindinium.org".to_string(),
    };

    let (url, obj) = training(&settings, 1, "m1".to_string());

    match vindinium::request_state(settings.key, url, obj) {
        None => (),
        Some(s) => println!("{}",s),
    } 

}
