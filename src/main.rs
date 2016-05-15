#![feature(core_intrinsics)]

extern crate glium;
mod universe;

use std::collections::HashSet;
use std::time::Instant;
use std::time::Duration;
use glium::DisplayBuild;
use glium::backend::glutin_backend::GlutinFacade;
use glium::glutin::Window;
use glium::glutin::VirtualKeyCode;
use glium::glutin::MouseButton;
use glium::Surface;
use universe::Universe;
use universe::Universe3D;

pub struct Simulation {
    universe: Box<Universe>,
    facade: Option<GlutinFacade>,
    start_instant: Option<Instant>,
    context: SimulationContext,
}

struct SimulationBuilder {
    universe: Option<Box<Universe>>,
}

impl Simulation {
    fn start(mut self) {
        let facade: GlutinFacade = glium::glutin::WindowBuilder::new()
            .with_dimensions(1024, 768)
            .with_title(format!("Hello world"))
            .build_glium()
            .unwrap();
        self.facade = Some(facade);
        self.start_instant = Some(Instant::now());

        self.render(self.start_instant.unwrap());

        'simulation: loop {
            self.context.update(self.facade.as_mut().unwrap());

        }
    }

    fn render(&mut self, now: Instant) {
        let mut frame = self.facade.as_mut().unwrap().draw();
        let start_instant = self.start_instant.unwrap();
        let time = now - start_instant;

        self.universe.render(&mut frame, &time);
        frame.finish();
    }

    fn builder() -> SimulationBuilder {
        SimulationBuilder {
            universe: None,
        }
    }
}

impl SimulationBuilder {
    fn universe(mut self, universe: Box<Universe>) -> SimulationBuilder {
        self.universe = Some(universe);
        self
    }

    fn build(self) -> Simulation {
        Simulation {
            universe: self.universe.unwrap(),
            facade: None,
            start_instant: None,
            context: SimulationContext::new(),
        }
    }
}

pub struct SimulationContext {
    pressed_keys: Vec<(u8, Option<VirtualKeyCode>)>,
    pressed_mouse_buttons: Vec<MouseButton>,
}

impl SimulationContext {
    fn new() -> SimulationContext {
        SimulationContext {
            pressed_keys: Vec::new(),
            pressed_mouse_buttons: Vec::new(),
        }
    }

    pub fn get_pressed_keys(&self) -> &Vec<(u8, Option<VirtualKeyCode>)> {
        &self.pressed_keys
    }

    pub fn get_pressed_mouse_buttons(&self) -> &Vec<MouseButton> {
        &self.pressed_mouse_buttons
    }

    pub fn update(&mut self, facade: &mut GlutinFacade) {
        for event in facade.poll_events() {
            print_type_of(&event);
        }
    }
}

fn main() {
    println!("Hello, world!");
    let simulation = Simulation::builder()
        .universe(Box::new(Universe3D::new()))
        .build();

    simulation.start();
}

/// A function for debugging purposes
fn print_type_of<T>(_: &T) -> () {
    let type_name =
        unsafe {
            std::intrinsics::type_name::<T>()
        };
    println!("{}", type_name);
}
