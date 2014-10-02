extern crate serialize;
use std::char::{to_digit};
use std::string::String;
use self::serialize::{Encoder, Encodable, Decoder, Decodable};
use self::serialize::json;

// Types

pub type Key = String;
pub type GameId = String;
pub type HeroId = int;

#[deriving(Show)]
pub struct Settings {
    pub key: Key,
    pub url: String,
}

#[deriving(Show)]
pub struct State {
    pub game: Game,
    pub hero: Hero,
    pub token: String,
    pub view_url: String,
    pub play_url: String,
}

#[deriving(Show)]
pub struct Game {
    pub id: GameId,
    pub turn: int,
    pub max_turns: int,
    pub heroes: int,
    pub board: Board,
    pub finished: bool,
}

#[deriving(Show)]
pub struct Pos {
    pub x: int,
    pub y: int,
}

#[deriving(Show)]
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

#[deriving(Show)]
pub struct Board {
    pub size: int,
    pub tiles: Vec<Tile>,
}

#[deriving(Show)]
pub enum Tile {
    FreeTile,
    WoodTile,
    TavernTile,
    HeroTile(HeroId),
    MineTile(Option<HeroId>),
}

#[deriving(Show)]
pub enum Dir {
    Stay,
    North,
    South,
    East,
    West,
}


// API

impl Settings {
    pub fn start_url(&self, v: &str) -> String {
        let mut url = self.url.clone();
        url.push_str("/api/");
        url.push_str(v);
        url
    }
}

pub fn request(settings: &Settings, val: &str) -> Option<State> {
    match json::decode(val) {
        Err(err) => {
           println!("err: vindinium::request - {}", err);
           None
        },
        Ok(state) => Some(state),
    }
}


// Json

impl <S: Encoder<E>, E> Encodable<S, E> for Dir {
    fn encode(&self, encoder: &mut S) -> Result<(), E> {
        match *self {
            Stay =>  encoder.emit_str("Stay"),
            North => encoder.emit_str("North"),
            South => encoder.emit_str("South"),
            East =>  encoder.emit_str("East"),
            West =>  encoder.emit_str("West"),
        }
    }
}

impl <S: Decoder<E>, E> Decodable<S, E> for Pos {
    fn decode(decoder: &mut S) -> Result<Pos, E> {
        decoder.read_struct("root", 0, |decoder| {
            Ok(Pos {
                x: try!(decoder.read_struct_field("x", 0, |decoder| Decodable::decode(decoder))),
                y: try!(decoder.read_struct_field("y", 1, |decoder| Decodable::decode(decoder))),
           })
        })
    }
}

impl <S: Decoder<E>, E> Decodable<S, E> for Hero {
    fn decode(decoder: &mut S) -> Result<Hero, E> {
        decoder.read_struct("root", 0, |decoder| {
            Ok(Hero {
                id: try!(decoder.read_struct_field("id", 0, |decoder| Decodable::decode(decoder))),
                name: try!(decoder.read_struct_field("name", 1, |decoder| Decodable::decode(decoder))),
                user_id: try!(decoder.read_struct_field("userId", 2, |decoder| {
                        Ok(match Decodable::decode(decoder) {
                            Ok(user_id) => Some(user_id),
                            Err(_) => None,
                        })
                    })),
                elo: try!(decoder.read_struct_field("elo", 3, |decoder| {
                        Ok( match Decodable::decode(decoder) {
                            Ok(elo) => Some(elo),
                            Err(_) => None,
                        })
                    })),
                pos: try!(decoder.read_struct_field("pos", 4, |decoder| Decodable::decode(decoder))),
                life: try!(decoder.read_struct_field("life", 5, |decoder| Decodable::decode(decoder))),
                gold: try!(decoder.read_struct_field("gold", 6, |decoder| Decodable::decode(decoder))),
                mine_count: try!(decoder.read_struct_field("mineCount", 7, |decoder| Decodable::decode(decoder))),
                spawn_pos: try!(decoder.read_struct_field("spawnPos", 8, |decoder| Decodable::decode(decoder))),
                crashed: try!(decoder.read_struct_field("crashed", 9, |decoder| Decodable::decode(decoder))),
            })
        })
    }
}

impl <S: Decoder<E>, E> Decodable<S, E> for Board {
    fn decode(decoder: &mut S) -> Result<Board, E> {
        let size = try!(decoder.read_struct_field("size", 0, |decoder| Decodable::decode(decoder)));
        Ok(Board { size: size, tiles: try!(decoder.read_struct_field("tiles", 1, |decoder| {
            let mut i = 0u;
            let mut tiles = Vec::new();
            let tile_str = try!(decoder.read_str());
            let tile_bytes = tile_str.as_bytes();
            loop {
                if i >= tile_str.len() {
                    break
                }
                match (tile_bytes[i] as char, tile_bytes[i+1] as char) {
                    (' ',' ') => tiles.push(FreeTile),
                    ('#','#') => tiles.push(WoodTile),
                    ('@',c)   => match to_digit(c,10) {
                        None => fail!("failed parse HeroTile num"),
                        Some(n) => tiles.push(HeroTile(n as int)),
                    },
                    ('[',']') => tiles.push(TavernTile),
                    ('$','-') => tiles.push(MineTile(None)),
                    ('$',c)   => match to_digit(c,10) {
                        None => fail!("failed parse MineTile num"),
                        Some(n) => tiles.push(MineTile(Some(n as int))),
                    },
                    (a,b) => fail!("failed parsing tile \"{}{}\"", a, b),
                }
                i += 2;
            }
            Ok(tiles)
        }))})
    }
}

impl <S: Decoder<E>, E> Decodable<S, E> for Game {
    fn decode(decoder: &mut S) -> Result<Game, E> {
        Ok(Game {
            id: try!(decoder.read_struct_field("id", 0, |decoder| Decodable::decode(decoder))),
            turn: try!(decoder.read_struct_field("turn", 1, |decoder| Decodable::decode(decoder))),
            max_turns: try!(decoder.read_struct_field("maxTurns",2, |decoder| Decodable::decode(decoder))),
            heroes: try!(decoder.read_struct_field("heroes", 3, |decoder| Decodable::decode(decoder))),
            board: try!(decoder.read_struct_field("board", 4, |decoder| Decodable::decode(decoder))),
            finished: try!(decoder.read_struct_field("finished", 5, |decoder| Decodable::decode(decoder))),
        })
    }
}

impl <S: Decoder<E>, E> Decodable<S, E> for State {
    fn decode(decoder: &mut S) -> Result<State, E> {
        Ok(State {
            game: try!(decoder.read_struct_field("game", 0, |decoder| Decodable::decode(decoder))),
            hero: try!(decoder.read_struct_field("hero", 0, |decoder| Decodable::decode(decoder))),
            token: try!(decoder.read_struct_field("token", 0, |decoder| Decodable::decode(decoder))),
            view_url: try!(decoder.read_struct_field("viewUrl", 0, |decoder| Decodable::decode(decoder))),
            play_url: try!(decoder.read_struct_field("playUrl", 0, |decoder| Decodable::decode(decoder))),
        })
    }
}
