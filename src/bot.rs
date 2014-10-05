use vindinium::{Bot, Dir, Stay, State};


pub struct RandomBot {
    pub dir: Dir,
}

impl Bot for RandomBot {
    fn move(&mut self, state: &State) -> Dir {
        self.dir = Stay;
        self.dir
    }
}
