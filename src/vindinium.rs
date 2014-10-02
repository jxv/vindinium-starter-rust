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
    pub heroes: Vec<Hero>,
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
    pub size: uint,
    pub tiles: Vec<Vec<Tile>>,
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
        Ok(Board { size: size, tiles: try!(d.read_struct_field("tiles", 1, |d| {
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
        }))})
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
