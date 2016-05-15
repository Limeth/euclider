extern crate glium;
mod universe;

use glium::DisplayBuild;
use universe::{Universe, Universe3D};

fn main() {
    println!("Hello, world!");
    let game = Game::builder()
        .with_universe(Box::new(Universe3D::new()))
        .build();

    game.start();
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
            universe: builder.universe.unwrap(),
        }
    }

    fn start(self) {
        let display = glium::glutin::WindowBuilder::new();
    }

    fn builder() -> GameBuilder {
        GameBuilder {
            universe: None,
        }
    }
}

impl GameBuilder {
    fn with_universe(mut self, universe: Box<Universe>) -> GameBuilder {
        self.universe = Some(universe);
        self
    }

    fn build(self) -> Game {
        Game::new(self)
    }
}
