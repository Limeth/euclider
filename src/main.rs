mod universe;

use universe::GDLUniverse;

fn main() {
    println!("Hello, world!");
}

pub struct Game {
    universe: Box<Universe>,
}

struct GameBuilder {
    universe: Option<Box<Universe>>,
}

impl Game {
    fn new(builder: GameBuilder) -> Game {
        Game {
            universe: builder.universe,
        }
    }

    fn builder() -> GameBuilder {
        GameBuilder {
            universe: None,
        }
    }
}

impl GameBuilder {
    fn with_universe(&mut self, universe: Universe) {
        self.universe = Box::new(universe);
    }
}
