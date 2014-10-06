extern crate serialize;
extern crate http;
extern crate url;
extern crate term;
use std::string::{String};
use std::fmt;
use std::collections::TreeMap;
use std::char::{to_digit};
use http::client::{RequestWriter};
use http::method::{Post};
use url::{Url};
use self::serialize::{Encoder, Encodable, Decoder, Decodable};
use self::serialize::json;
use self::term::{Terminal};
use self::term::color;


// Types

pub type Key = String;
pub type GameId = String;
pub type HeroId = int;

#[deriving(Show, Clone)]
pub enum Mode {
    Training(Option<u64>,Option<String>),
    Arena
}

#[deriving(Show, Clone)]
pub struct Settings {
    pub key: Key,
    pub url: String,
    pub mode: Mode,
}

#[deriving(Show, Clone)]
pub struct State {
    pub game: Game,
    pub hero: Hero,
    pub token: String,
    pub view_url: String,
    pub play_url: String,
}

#[deriving(Show, Clone)]
pub struct Game {
    pub id: GameId,
    pub turn: int,
    pub max_turns: int,
    pub heroes: Vec<Hero>,
    pub board: Board,
    pub finished: bool,
}

#[deriving(Show, Clone)]
pub struct Pos {
    pub x: int,
    pub y: int,
}

#[deriving(Show, Clone)]
pub struct Hero {
    pub id: HeroId,
    pub name: String,
    pub user_id: Option<String>,
    pub elo: Option<int>,
    pub pos: Pos,
    pub life: int,
    pub gold: int,
    pub mine_count: int,
    pub spawn_pos: Pos,
    pub crashed: bool,
}

#[deriving(Show, Clone)]
pub struct Board {
    pub size: uint,
    pub tiles: Vec<Vec<Tile>>,
}

#[deriving(Show, Clone)]
pub enum Tile {
    FreeTile,
    WoodTile,
    TavernTile,
    HeroTile(HeroId),
    MineTile(Option<HeroId>),
}

#[deriving(Show, Clone, Rand)]
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

pub fn request(url: String, obj: json::JsonObject) -> Option<State> {
    let url = match Url::parse(url.as_slice()) {
        Ok(u) => u,
        Err(err) => {
            println!("{}", err);
            return None
        },
    };

    let mut request: RequestWriter = match RequestWriter::new(Post, url) {
        Ok(req) => req,
        Err(err) => {
            println!("{}", err);
            return None
        },
    };
    
    let msg = json::encode(&json::Object(obj));
    let content_type = http::headers::content_type::MediaType::new
        ("application".to_string(), "json".to_string(), Vec::new());
    request.headers.content_length = Some(msg.len());
    request.headers.content_type = Some(content_type);
    request.headers.accept = Some("application/json".to_string());
    request.headers.user_agent = Some("vindinium-starter-rust".to_string());

    match request.write(msg.as_bytes()) {
        Ok(()) => (),
        Err(err) => {
            println!("{}", err);
            return None
        }
    };

    let mut response = match request.read_response() {
        Ok(resp) => resp,
        Err((_, err)) => {
            println!("{}", err);
            return None
        }
    };
    let state_str = match response.read_to_string() {
        Ok(s) => s,
        Err(err) => {
            println!("{}", err);
            return None
        }
    };

    match json::decode(state_str.as_slice()) {
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
}

pub fn step_msg(settings: &Settings, state: &State, dir: Dir) -> (String, json::JsonObject) {
    let mut obj: json::JsonObject = TreeMap::new();
    obj.insert("key".to_string(), json::String(settings.key.clone()));
    obj.insert("dir".to_string(), json::String(format_args!(fmt::format, "{}", dir))); 
    (state.play_url.clone(), obj)
}

pub fn start_msg(settings: &Settings) -> (String, json::JsonObject) {
    match settings.mode.clone() {
        Training(opt_turns, opt_map) => start_training_msg(settings, opt_turns, opt_map),
        Arena => start_arena_msg(settings),
    }
}

pub fn start_training_msg(settings: &Settings, opt_turns: Option<u64>, opt_map: Option<String>) -> (String, json::JsonObject) {
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

pub fn start_arena_msg(settings: &Settings) -> (String, json::JsonObject) {
    let mut obj: json::JsonObject = TreeMap::new();
    obj.insert("key".to_string(), json::String(settings.key.clone()));
    (settings.start_url("arena"), obj)
}

// Json

impl <S: Encoder<E>, E> Encodable<S, E> for Dir {
    fn encode(&self, e: &mut S) -> Result<(), E> {
        match *self {
            Stay =>  e.emit_str("Stay"),
            North => e.emit_str("North"),
            South => e.emit_str("South"),
            East =>  e.emit_str("East"),
            West =>  e.emit_str("West"),
        }
    }
}

impl <S: Decoder<E>, E> Decodable<S, E> for Pos {
    fn decode(d: &mut S) -> Result<Pos, E> {
        d.read_struct("root", 0, |d| {
            Ok(Pos {
                x: try!(d.read_struct_field("x", 0, |d| Decodable::decode(d))),
                y: try!(d.read_struct_field("y", 1, |d| Decodable::decode(d))),
           })
        })
    }
}

impl <S: Decoder<E>, E> Decodable<S, E> for Hero {
    fn decode(d: &mut S) -> Result<Hero, E> {
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

impl <S: Decoder<E>, E> Decodable<S, E> for Board {
    fn decode(d: &mut S) -> Result<Board, E> {
        let size = try!(d.read_struct_field("size", 0, |d| Decodable::decode(d)));
        let tiles = try!(d.read_struct_field("tiles", 1, |d| {
            let mut tiles: Vec<Vec<Tile>> = Vec::with_capacity(size);
            let tile_str = try!(d.read_str());
            if tile_str.len() != size * size * 2 {
                return Err(d.error("tile string is incorrect size"));
            }
            let tile_bytes = tile_str.as_bytes();
            let mut i = 0u;
            loop {
                let mut row: Vec<Tile> = Vec::with_capacity(size);
                loop {
                    if i >= tile_str.len() {
                        break;
                    }
                    match (tile_bytes[i] as char, tile_bytes[i+1] as char) {
                        (' ',' ') => row.push(FreeTile),
                        ('#','#') => row.push(WoodTile),
                        ('@',c)   => match to_digit(c,10) {
                            None => return Err(d.error("failed parse HeroTile num")),
                            Some(n) => row.push(HeroTile(n as int)),
                        },
                        ('[',']') => row.push(TavernTile),
                        ('$','-') => row.push(MineTile(None)),
                        ('$',c)   => match to_digit(c,10) {
                            None => return Err(d.error("failed parse MineTile num")),
                            Some(n) => row.push(MineTile(Some(n as int))),
                        },
                        (a,b) => return Err(d.error(format!("failed parsing tile \"{}{}\"", a, b).as_slice())),
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

impl <S: Decoder<E>, E> Decodable<S, E> for Game {
    fn decode(d: &mut S) -> Result<Game, E> {
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

impl <S: Decoder<E>, E> Decodable<S, E> for State {
    fn decode(d: &mut S) -> Result<State, E> {
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
        for _ in range(0, self.game.board.size) {
            print!("\x1b[1A\x1b[2K");
        }
        // clear players info
        for _ in range(0,self.game.heroes.len()-1) {
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
        for &ref row in self.game.board.tiles.iter() {
            for &tile in row.iter() {
                let s: String = match tile {
                    FreeTile => {
                        term.bg(color::BRIGHT_BLACK).unwrap();
                        "  ".to_string()
                    },
                    WoodTile => {
                        term.bg(color::BRIGHT_BLACK).unwrap();
                        term.fg(color::WHITE).unwrap();
                        "##".to_string()
                    },
                    TavernTile => {
                        term.bg(color::BRIGHT_BLACK).unwrap();
                        term.fg(color::WHITE).unwrap();
                        "[]".to_string()
                    },
                    HeroTile(hero_id) => {
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
                    MineTile(None) => {
                        term.bg(color::BRIGHT_BLACK).unwrap();
                        term.fg(color::WHITE).unwrap();
                        "$-".to_string()
                    },
                    MineTile(Some(hero_id)) => {
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
        for i in range(0, self.game.heroes.len()) {
            let ref hero = self.game.heroes[i];
            term.fg(match i {
                0 => color::BRIGHT_RED,
                1 => color::BRIGHT_BLUE,
                2 => color::BRIGHT_GREEN,
                3 => color::BRIGHT_YELLOW,
                _ => color::WHITE,
            }).unwrap();
            (writeln!(term,"@1\t{}\tlife:{}\tmines:{}\tgold:{}",
                hero.name, hero.life, hero.mine_count, hero.gold)).unwrap();
        }
        // reset colors to back default
        term.reset().unwrap();
    }
}
