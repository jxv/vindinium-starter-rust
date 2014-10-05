use std::rand;
use vindinium::{Bot, Dir, Stay, State, HeroId};


#[deriving(Show,Clone)]
pub struct RandomBot {
    pub hero_id: HeroId,
    pub dir: Dir,
}

impl RandomBot {
    pub fn new() -> RandomBot {
        RandomBot {
            hero_id: 0,
            dir: Stay,
        }
    }
}

impl Bot for RandomBot {

    fn step(&self, state: &State) -> RandomBot {
        println!("{}\n\n", state);
        let mut bot: RandomBot = self.clone();
        bot.hero_id = state.hero.id;
        bot.dir = rand::random();
        bot 
    }

    fn dir(&self) -> Dir {
        self.dir
    }
}
