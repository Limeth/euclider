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
use glium::glutin::ElementState;
use glium::glutin::Event;
use glium::Surface;
use universe::Universe;
use universe::Universe3D;

pub struct Simulation<U: Universe> {
    universe: Box<U>,
    facade: Option<GlutinFacade>,
    start_instant: Option<Instant>,
    last_updated_instant: Option<Instant>,
    context: SimulationContext,
}

struct SimulationBuilder<U: Universe> {
    universe: Option<Box<U>>,
}

impl<U: Universe> Simulation<U> {
    fn start(mut self) {
        let facade: GlutinFacade = glium::glutin::WindowBuilder::new()
            .with_dimensions(1024, 768)
            .with_title(format!("Hello world"))
            .build_glium()
            .unwrap();
        self.facade = Some(facade);
        self.start_instant = Some(Instant::now());

        self.render();

        'simulation: loop {
            match self.update() {
                Err(_) => break 'simulation,
                _ => (),
            }

            self.render();
        }
    }

    fn render(&mut self) {
        let mut frame = self.facade.as_mut().unwrap().draw();
        let start_instant = self.start_instant.unwrap();
        let time = Instant::now() - start_instant;

        self.universe.render(&mut frame, &time);
        frame.finish();
    }

    fn update(&mut self) -> Result<(), ()> {
        let delta: Duration;
        let now = Instant::now();

        if self.last_updated_instant.is_none() {
            delta = now - self.start_instant.unwrap();
        } else {
            delta = now - self.last_updated_instant.unwrap();
        }

        self.universe.update(&delta);

        if self.context.pressed_keys.iter()
            .any(|&tuple| tuple.1.is_some() && tuple.1.unwrap() == VirtualKeyCode::Escape) {
            return Result::Err(());
        }

        println!("{:?}", self.context);

        self.context.update(self.facade.as_mut().unwrap());
        Result::Ok(())
    }

    fn builder() -> SimulationBuilder<U> {
        SimulationBuilder {
            universe: None,
        }
    }
}

impl<U: Universe> SimulationBuilder<U> {
    fn universe(mut self, universe: Box<U>) -> SimulationBuilder<U> {
        self.universe = Some(universe);
        self
    }

    fn build(self) -> Simulation<U> {
        Simulation {
            universe: self.universe.unwrap(),
            facade: None,
            start_instant: None,
            last_updated_instant: None,
            context: SimulationContext::new(),
        }
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct SimulationContext {
    pressed_keys: HashSet<(u8, Option<VirtualKeyCode>)>,
    pressed_mouse_buttons: HashSet<MouseButton>,
}

impl SimulationContext {
    fn new() -> SimulationContext {
        SimulationContext {
            pressed_keys: HashSet::new(),
            pressed_mouse_buttons: HashSet::new(),
        }
    }

    pub fn get_pressed_keys(&self) -> &HashSet<(u8, Option<VirtualKeyCode>)> {
        &self.pressed_keys
    }

    pub fn get_pressed_mouse_buttons(&self) -> &HashSet<MouseButton> {
        &self.pressed_mouse_buttons
    }

    pub fn update(&mut self, facade: &mut GlutinFacade) {
        for event in facade.poll_events() {
            match event {
                Event::KeyboardInput(state, character, virtual_code) => {
                    match state {
                        ElementState::Pressed => {
                            self.pressed_keys.insert((character, virtual_code));
                        },
                        ElementState::Released => {
                            self.pressed_keys.remove_if(|tuple: &(u8, Option<VirtualKeyCode>)| {
                                tuple.0 == character
                            });
                        },
                    };
                },
                Event::MouseInput(state, button) => {

                },
                _ => (),
            }
            print_type_of(&event);
        }
    }
}

trait RemoveIf<T, C> {
    fn remove_if<F>(&mut self, f: F) -> C where F: Fn(&T) -> bool;
}

use std::hash::Hash;

impl<T> RemoveIf<T, HashSet<T>> for HashSet<T> where T: Eq + Copy + Hash {
    fn remove_if<F>(&mut self, f: F) -> HashSet<T> where F: Fn(&T) -> bool {
        let mut removed: HashSet<T> = HashSet::new();

        for value in self.iter() {
            if f(value) {
                removed.insert(value.clone());
            }
        }

        for removed_value in &removed {
            self.remove(&removed_value);
        }

        removed
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
