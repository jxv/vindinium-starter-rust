extern crate hyper;
extern crate url;
extern crate term;
extern crate rustc_serialize;
use std::string::{String};
use std::io::Read;
use std::fmt;
use std::collections::BTreeMap;
use std::char;
use hyper::client::Client;
use hyper::header::{ContentLength, ContentType, Accept, UserAgent, qitem};
use hyper::mime::Mime;
use url::{Url};
use rustc_serialize::json;
use rustc_serialize::json::Json;
use rustc_serialize::{Encoder, Encodable, Decoder, Decodable};
use self::term::{Terminal};
use self::term::color;

// Types

pub type Key = String;
pub type GameId = String;
pub type HeroId = isize;

#[derive(Debug, Clone)]
pub enum Mode {
    Training(Option<u64>,Option<String>),
    Arena
}

#[derive(Debug, Clone)]
pub struct Settings {
    pub key: Key,
    pub url: String,
    pub mode: Mode,
}

#[derive(Debug, Clone)]
pub struct State {
    pub game: Game,
    pub hero: Hero,
    pub token: String,
    pub view_url: String,
    pub play_url: String,
}

#[derive(Debug, Clone)]
pub struct Game {
    pub id: GameId,
    pub turn: isize,
    pub max_turns: isize,
    pub heroes: Vec<Hero>,
    pub board: Board,
    pub finished: bool,
}

#[derive(Debug, Clone)]
pub struct Pos {
    pub x: isize,
    pub y: isize,
}

#[derive(Debug, Clone)]
pub struct Hero {
    pub id: HeroId,
    pub name: String,
    pub user_id: Option<String>,
    pub elo: Option<isize>,
    pub pos: Pos,
    pub life: isize,
    pub gold: isize,
    pub mine_count: isize,
    pub spawn_pos: Pos,
    pub crashed: bool,
}

#[derive(Debug, Clone)]
pub struct Board {
    pub size: usize,
    pub tiles: Vec<Vec<Tile>>,
}

#[derive(Debug, Clone)]
pub enum Tile {
    Free,
    Wood,
    Tavern,
    Hero(HeroId),
    Mine(Option<HeroId>),
}

#[derive(Debug, Clone)]
pub enum Dir {
    Stay,
    North,
    South,
    East,
    West,
}


// API

pub trait Bot {
    fn step(&self, &State) -> Self;
    fn dir(&self) -> Dir;
}

impl Settings {
    pub fn start_url(&self, v: &str) -> String {
        let mut url = self.url.clone();
        url.push_str("/api/");
        url.push_str(v);
        url
    }
}

fn parse_request(url: Url, obj: json::Object) -> Option<State> {
    let content_type: Mime = "Application/Json".parse().unwrap();
    let client = Client::new();
    let msg = json::encode(&obj).unwrap();
    let request = client.post(url).body(&msg)
        .header(ContentLength(msg.len() as u64))
        .header(ContentType(content_type.clone()))
        .header(Accept(vec![qitem(content_type)]))
        .header(UserAgent("vindinium-starter-rust".to_string()));

    let mut response = request.send().unwrap();
    assert_eq!(response.status, hyper::Ok);

    let mut state_str = String::new();
    match response.read_to_string(&mut state_str) {
        Ok(_) => {
            return match json::decode(&state_str) {
                Ok(state) => Some(state),
                Err(err) => {
                    if "Vindinium - The game is finished".to_string() == state_str {
                        println!("Timeout!");
                    } else {
                        println!("{}", err);
                    }
                    None
                },
            }
        },
        Err(err) => {
            println!("{}", err);
            return None
        }
    };
}

pub fn request(url: String, obj: json::Object) -> Option<State> {
    match Url::parse(url.as_str()) {
        Ok(u) => return parse_request(u, obj),
        Err(err) => {
            println!("{}", err);
            return None
        }
    };
}

pub fn step_msg(settings: &Settings, state: &State, dir: Dir) -> (String, json::Object) {
    let mut obj: json::Object = json::Object::new();
    obj.insert("key".to_string(), Json::String(settings.key.clone()));
    obj.insert("dir".to_string(), Json::String(dir.to_string()));
    (state.play_url.clone(), obj)
}

pub fn start_msg(settings: &Settings) -> (String, json::Object) {
    match settings.mode.clone() {
        Mode::Training(opt_turns, opt_map) => start_training_msg(settings, opt_turns, opt_map),
        Mode::Arena => start_arena_msg(settings),
    }
}

pub fn start_training_msg(settings: &Settings, opt_turns: Option<u64>, opt_map: Option<String>) -> (String, json::Object) {
    let mut obj: json::Object = BTreeMap::new();
    obj.insert("key".to_string(), Json::String(settings.key.clone()));
    match opt_turns {
        Some(turns) => { obj.insert("turns".to_string(), Json::U64(turns)); },
        None => (),
    };
    match opt_map {
        Some(map) => { obj.insert("map".to_string(), Json::String(map)); },
        None => (),
    };
    (settings.start_url("training"), obj)
}

pub fn start_arena_msg(settings: &Settings) -> (String, json::Object) {
    let mut obj: json::Object = BTreeMap::new();
    obj.insert("key".to_string(), Json::String(settings.key.clone()));
    (settings.start_url("arena"), obj)
}

// Json

impl Encodable for Dir {
    fn encode<S: Encoder>(&self, s: &mut S) -> Result<(), S::Error> {
        match *self {
            Dir::Stay =>  s.emit_str("Stay"),
            Dir::North => s.emit_str("North"),
            Dir::South => s.emit_str("South"),
            Dir::East =>  s.emit_str("East"),
            Dir::West =>  s.emit_str("West"),
        }
    }
}

impl Decodable for Pos {
    fn decode<D: Decoder>(d: &mut D) -> Result<Self, D::Error> {
        d.read_struct("root", 0, |d| {
            Ok(Pos {
                x: try!(d.read_struct_field("x", 0, |d| Decodable::decode(d))),
                y: try!(d.read_struct_field("y", 1, |d| Decodable::decode(d))),
           })
        })
    }
}

impl Decodable for Hero {
    fn decode<D: Decoder>(d: &mut D) -> Result<Hero, D::Error> {
        d.read_struct("root", 0, |d| {
            Ok(Hero {
                id: try!(d.read_struct_field("id", 0, |d| Decodable::decode(d))),
                name: try!(d.read_struct_field("name", 1, |d| Decodable::decode(d))),
                user_id: try!(d.read_struct_field("userId", 2, |d| {
                    Ok(match Decodable::decode(d) {
                        Ok(user_id) => Some(user_id),
                        Err(_) => None,
                    })
                })),
                elo: try!(d.read_struct_field("elo", 3, |d| {
                    Ok( match Decodable::decode(d) {
                        Ok(elo) => Some(elo),
                         Err(_) => None,
                    })
                })),
                pos: try!(d.read_struct_field("pos", 4, |d| Decodable::decode(d))),
                life: try!(d.read_struct_field("life", 5, |d| Decodable::decode(d))),
                gold: try!(d.read_struct_field("gold", 6, |d| Decodable::decode(d))),
                mine_count: try!(d.read_struct_field("mineCount", 7, |d| Decodable::decode(d))),
                spawn_pos: try!(d.read_struct_field("spawnPos", 8, |d| Decodable::decode(d))),
                crashed: try!(d.read_struct_field("crashed", 9, |d| Decodable::decode(d))),
            })
        })
    }
}

impl Decodable for Board {
    fn decode<D: Decoder>(d: &mut D) -> Result<Board, D::Error> {
        let size = try!(d.read_struct_field("size", 0, |d| Decodable::decode(d)));
        let tiles = try!(d.read_struct_field("tiles", 1, |d| {
            let mut tiles: Vec<Vec<Tile>> = Vec::with_capacity(size);
            let tile_str = try!(d.read_str());
            if tile_str.len() != size * size * 2 {
                return Err(d.error("tile string is incorrect size"));
            }
            let tile_bytes = tile_str.as_bytes();
            let mut i = 0;
            loop {
                let mut row: Vec<Tile> = Vec::with_capacity(size);
                loop {
                    if i >= tile_str.len() {
                        break;
                    }
                    match (tile_bytes[i] as char, tile_bytes[i+1] as char) {
                        (' ',' ') => row.push(Tile::Free),
                        ('#','#') => row.push(Tile::Wood),
                        ('@',c)   => match char::to_digit(c,10) {
                            None => return Err(d.error("failed parse Tile::Hero num")),
                            Some(n) => row.push(Tile::Hero(n as isize)),
                        },
                        ('[',']') => row.push(Tile::Tavern),
                        ('$','-') => row.push(Tile::Mine(None)),
                        ('$',c)   => match char::to_digit(c,10) {
                            None => return Err(d.error("failed parse Tile::Mine num")),
                            Some(n) => row.push(Tile::Mine(Some(n as isize))),
                        },
                        (a,b) => return Err(d.error(&format!("failed parsing tile \"{}{}\"", a, b))),
                    }
                    i += 2;
                    if i % (size * 2) == 0 {
                        break;
                    }
                }
                tiles.push(row);
                if i >= tile_str.len() {
                    break;
                }
            }
            Ok(tiles)
        }));
        Ok(Board { size: size, tiles: tiles })
    }
}

impl Decodable for Game {
    fn decode<D: Decoder>(d: &mut D) -> Result<Game, D::Error>{
        Ok(Game {
            id: try!(d.read_struct_field("id", 0, |d| Decodable::decode(d))),
            turn: try!(d.read_struct_field("turn", 1, |d| Decodable::decode(d))),
            max_turns: try!(d.read_struct_field("maxTurns",2, |d| Decodable::decode(d))),
            heroes: try!(d.read_struct_field("heroes", 3, |d| Decodable::decode(d))),
            board: try!(d.read_struct_field("board", 4, |d| Decodable::decode(d))),
            finished: try!(d.read_struct_field("finished", 5, |d| Decodable::decode(d))),
        })
    }
}

impl Decodable for State {
    fn decode<D: Decoder>(d: &mut D) -> Result<State, D::Error> {
        Ok(State {
            game: try!(d.read_struct_field("game", 0, |d| Decodable::decode(d))),
            hero: try!(d.read_struct_field("hero", 1, |d| Decodable::decode(d))),
            token: try!(d.read_struct_field("token", 2, |d| Decodable::decode(d))),
            view_url: try!(d.read_struct_field("viewUrl", 3, |d| Decodable::decode(d))),
            play_url: try!(d.read_struct_field("playUrl", 4, |d| Decodable::decode(d))),
        })
    }
}

// Misc.

impl State {
    pub fn clear_pretty_print(&self) {
        // clear game info
        print!("\x1b[1A\x1b[2K");
        // clear board 
        for _ in 0..self.game.board.size {
            print!("\x1b[1A\x1b[2K");
        }
        // clear players info
        for _ in 0..self.game.heroes.len()-1 {
            print!("\x1b[1A\x1b[2K");
        }
        // clear last player info without clearing an extra line
        println!("\x1b[1A\x1b[2K\x1b[1A");
    }
    pub fn pretty_print(&self) {
        let mut term = term::stdout().unwrap();
        // print game info
        (writeln!(term, "id:{} turns:{}/{}", self.game.id, self.game.turn, self.game.max_turns)).unwrap();
        // print tiles on board
        for row in &self.game.board.tiles {
            for tile in row {
                let s: String = match *tile {
                    Tile::Free => {
                        term.bg(color::BRIGHT_BLACK).unwrap();
                        "  ".to_string()
                    },
                    Tile::Wood => {
                        term.bg(color::BRIGHT_BLACK).unwrap();
                        term.fg(color::WHITE).unwrap();
                        "##".to_string()
                    },
                    Tile::Tavern => {
                        term.bg(color::BRIGHT_BLACK).unwrap();
                        term.fg(color::WHITE).unwrap();
                        "[]".to_string()
                    },
                    Tile::Hero(hero_id) => {
                        term.bg(color::BRIGHT_BLACK).unwrap();
                        match hero_id {
                            1 => {
                                term.fg(color::BRIGHT_RED).unwrap();
                                "@1".to_string()
                            }
                            2 => {
                                term.fg(color::BRIGHT_BLUE).unwrap();
                                "@2".to_string()
                            }
                            3 => {
                                term.fg(color::BRIGHT_GREEN).unwrap();
                                "@3".to_string()
                            }
                            4 => {
                                term.fg(color::BRIGHT_YELLOW).unwrap();
                                "@4".to_string()
                            }
                            _ => {
                                "  ".to_string()
                            }
                        }
                    },
                    Tile::Mine(None) => {
                        term.bg(color::BRIGHT_BLACK).unwrap();
                        term.fg(color::WHITE).unwrap();
                        "$-".to_string()
                    },
                    Tile::Mine(Some(hero_id)) => {
                        term.fg(color::WHITE).unwrap();
                        match hero_id {
                            1 => {
                                term.bg(color::RED).unwrap();
                                "$1".to_string()
                            }
                            2 => {
                                term.bg(color::BLUE).unwrap();
                                "$2".to_string()
                            }
                            3 => {
                                term.bg(color::GREEN).unwrap();
                                "$3".to_string()
                            }
                            4 => {
                                term.bg(color::YELLOW).unwrap();
                                "$4".to_string()
                            }
                            _ => {
                                "  ".to_string()
                            }
                        }
                    },
                };
                (write!(term, "{}", s)).unwrap();
            }
            (writeln!(term,"")).unwrap();
        }
        term.reset().unwrap();
        // print players' info
        for i in 0..self.game.heroes.len() {
            let ref hero = self.game.heroes[i];
            term.fg(match i {
                0 => color::BRIGHT_RED,
                1 => color::BRIGHT_BLUE,
                2 => color::BRIGHT_GREEN,
                3 => color::BRIGHT_YELLOW,
                _ => color::WHITE,
            }).unwrap();
            (writeln!(term,"@{}\t{}\tlife:{}\tmines:{}\tgold:{}",
               i+1, hero.name, hero.life, hero.mine_count, hero.gold)).unwrap();
        }
        // reset colors to back default
        term.reset().unwrap();
    }
}

impl fmt::Display for Dir {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Dir::Stay  => write!(f, "{}", "Stay"),
            Dir::North => write!(f, "{}", "North"),
            Dir::South => write!(f, "{}", "South"),
            Dir::East  => write!(f, "{}", "East"),
            Dir::West  => write!(f, "{}", "West"),
        }
    }
}
