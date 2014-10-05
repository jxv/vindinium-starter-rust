use std::rand;
use vindinium::{Bot, Dir, State, HeroId};


pub struct RandomBot {
    pub hero_id: HeroId,
}

impl RandomBot {
    pub fn new() -> RandomBot {
        RandomBot { hero_id: 0 }
    }
}

impl Bot for RandomBot {

    fn step(&mut self, state: &State) -> Dir {
        println!("{}\n\n", state);
        self.hero_id = state.hero.id;
        rand::random()
    }
}
