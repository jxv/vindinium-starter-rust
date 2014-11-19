use std::rand;
use vindinium::{Bot, Dir, State};


#[deriving(Show,Clone)]
pub struct RandomBot {
    pub dir: Dir,
}

impl RandomBot {
    pub fn new() -> RandomBot {
        RandomBot {
            dir: Dir::Stay,
        }
    }
}

impl Bot for RandomBot {

    fn step(&self, _: &State) -> RandomBot {
        let mut bot: RandomBot = self.clone();
        bot.dir = rand::random();
        bot 
    }

    fn dir(&self) -> Dir {
        self.dir
    }
}
