#![feature(globs)]
extern crate serialize;
extern crate http;
extern crate url;
use std::string::String;
use std::io::File;
use self::serialize::json;
use http::client::RequestWriter;
use http::method::{Get,Post};
use url::Url;

use vindinium::*;
mod vindinium;


fn main() {
    let url = Url::parse("http://vindinium.org/api/training").unwrap();
    let mut request: RequestWriter = match RequestWriter::new(Post, url) {
        Ok(req) => req,
        Err(err) => fail!(":-( {}", err),
    };
    let data = b"{\"key\":\"MY_KEY\", \"turns\":1, \"map\":\"m1\"}";
    let json_mt = http::headers::content_type::MediaType::new(String::from_str("application"),String::from_str("json"),Vec::new());
    request.headers.content_length = Some(data.len());
    request.headers.content_type = Some(json_mt);
    request.headers.accept = Some(String::from_str("application/json"));
    request.headers.user_agent = Some(String::from_str("vindinium-starter-rust"));
    request.write(data);
    //println!("{}",request);

    let mut response = match request.read_response() {
        Ok(resp) => resp,
        Err((req, err)) => fail!(":-( {}", err),
    };
    let state_str = match response.read_to_string() {
        Ok(s) => s,
        Err(err) => fail!("{}", err),
    };
    println!("{}",state_str);

    let contents = File::open(&Path::new("example.json")).read_to_string();
    let data = match contents {
        Ok(d) => d,
        Err(_) => String::from_str(""),
    };
    // println!("{}", data);

    let settings = vindinium::Settings {
        key: String::from_str(""),
        url: String::from_str("")
    };

    // match vindinium::request(&settings, data.as_slice()) {
    match vindinium::request(&settings, state_str.as_slice()) {
        None => (),
        Some(s) => println!("{}",s),
    } 

    // println!("{}", json::encode(&North));

/*
    {
        let j = "{\"x\":8, \"y\":9}";
        let res: json::DecodeResult<Pos> = json::decode(j);
        match res {
            Ok(pos) => println!("{}", pos),
            Err(err) => println!("err: {}",err),
        };
    }
*/

/*
    {
        let j = "{\"id\":1, \"name\":\"bobby\", \"userId\":\"bobbert\", \"elo\":10, \"pos\":{\"x\":10,\"y\":80}, \"life\":50, \"gold\":700, \"mineCount\":3, \"spawnPos\":{\"x\":8,\"y\":3}, \"crashed\":false}";
        let res: json::DecodeResult<Hero> = json::decode(j);
        match res {
            Ok(hero) => println!("{}", hero),
            Err(err) => println!("err: {}",err),
        };
    }
*/
/*
    {
        let j = "{\"size\":4, \"tiles\":\"@1  ##[]@2  $-$2@3  []  @4##  ##$-\"}";
        let res: json::DecodeResult<Board> = json::decode(j);
        match res {
            Ok(board) => println!("{}", board),
            Err(err) => println!("err: {}",err),
        };
    }
*/
}
