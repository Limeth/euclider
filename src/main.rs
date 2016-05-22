#![feature(core_intrinsics)]

extern crate nalgebra as na;
extern crate glium;
mod universe;
pub mod util;

use std::collections::HashSet;
use std::time::Instant;
use std::time::Duration;
use na::*;
use glium::DisplayBuild;
use glium::backend::glutin_backend::GlutinFacade;
use glium::glutin::VirtualKeyCode;
use glium::glutin::MouseButton;
use glium::glutin::ElementState;
use glium::glutin::Event;
use glium::glutin::CursorState;
use universe::Universe;
use universe::d3::Universe3D;
use util::RemoveIf;

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

        facade.get_window().unwrap().set_cursor_state(CursorState::Hide);

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

        self.universe.render(self.facade.as_ref().unwrap(), &mut frame, &time, &self.context);

        match frame.finish() {
            Err(error) => panic!("An error occured while swapping the OpenGL buffers: {:?}", error),
            _ => (),
        }
    }

    fn update(&mut self) -> Result<(), Event> {
        let delta: Duration;
        let now = Instant::now();

        if self.last_updated_instant.is_none() {
            delta = now - self.start_instant.unwrap();
        } else {
            delta = now - self.last_updated_instant.unwrap();
        }

        let result = self.context.update(self.facade.as_mut().unwrap());
        self.universe.update(&delta, &self.context);

        result
    }

    fn builder() -> SimulationBuilder<U> {
        SimulationBuilder {
            universe: None,
        }
    }
}

impl<U: Universe> SimulationBuilder<U> {
    fn universe(mut self, universe: U) -> SimulationBuilder<U> {
        self.universe = Some(Box::new(universe));
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
    mouse: Point2<i32>,
    delta_mouse: Vector2<i32>,
}

impl SimulationContext {
    fn new() -> SimulationContext {
        SimulationContext {
            pressed_keys: HashSet::new(),
            pressed_mouse_buttons: HashSet::new(),
            mouse: na::origin(),
            delta_mouse: na::zero(),
        }
    }

    pub fn pressed_keys(&self) -> &HashSet<(u8, Option<VirtualKeyCode>)> {
        &self.pressed_keys
    }

    pub fn pressed_mouse_buttons(&self) -> &HashSet<MouseButton> {
        &self.pressed_mouse_buttons
    }

    pub fn delta_mouse(&self) -> Vector2<i32> {
        self.delta_mouse
    }

    fn reset_delta_mouse(&mut self) {
        self.delta_mouse = na::zero();
    }

    pub fn update(&mut self, facade: &mut GlutinFacade) -> Result<(), Event> {
        self.reset_delta_mouse();
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
                            if virtual_code.map_or(false, |virtual_code| {
                                virtual_code == VirtualKeyCode::Escape
                            }) {
                                return Err(event);
                            }
                        },
                    };
                },
                Event::MouseInput(state, button) => {
                    match state {
                        ElementState::Pressed => {
                            self.pressed_mouse_buttons.insert(button);
                        },
                        ElementState::Released => {
                            self.pressed_mouse_buttons.remove(&button);
                        },
                    }
                },
                Event::MouseMoved(x, y) => {
                    let window = facade.get_window().unwrap();
                    let window_size = window.get_inner_size_pixels().unwrap();
                    let center_x = window_size.0 as i32 / 2;
                    let center_y = window_size.1 as i32 / 2;

                    self.delta_mouse.x = self.mouse.x - x;
                    self.delta_mouse.y = self.mouse.y - y;
                    self.mouse.x = x;
                    self.mouse.y = y;

                    window.set_cursor_position(center_x, center_y).expect("Could not reset the cursor position.");
                },
                Event::Closed => {
                    return Err(event);
                },
                _ => (),
            }
        };
        Ok(())
    }
}

fn main() {
    let simulation = Simulation::builder()
        .universe(Universe3D::new())
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
