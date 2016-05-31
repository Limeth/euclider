#![feature(reflect_marker)]
#![feature(custom_attribute)]

extern crate nalgebra as na;
extern crate scoped_threadpool;
extern crate image;
extern crate noise;
extern crate rand;
extern crate palette;
extern crate glium;
mod universe;
pub mod util;

use std::collections::HashSet;
use std::time::Instant;
use std::time::Duration;
use rand::StdRng;
use na::Point3;
use na::Point2;
use na::Vector3;
use na::Vector2;
use glium::DisplayBuild;
use glium::backend::glutin_backend::GlutinFacade;
use glium::glutin::VirtualKeyCode;
use glium::glutin::MouseButton;
use glium::glutin::ElementState;
use glium::glutin::Event;
use glium::glutin::CursorState;
use glium::glutin::MouseScrollDelta;
use image::Rgba;
use universe::Universe;
use universe::entity::*;
use universe::d3::Universe3D;
use universe::d3::entity::*;
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

        facade.get_window().unwrap().set_cursor_state(CursorState::Hide)
            .expect("Could not hide the cursor!");

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
        let now = Instant::now();
        let time = now - self.start_instant.unwrap();

        self.universe.render(self.facade.as_ref().unwrap(),
                             &mut frame,
                             &time,
                             &self.context);

        match frame.finish() {
            Err(error) => {
                panic!("An error occured while swapping the OpenGL buffers: {:?}",
                       error)
            }
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

        self.last_updated_instant = Some(now);

        let result = self.context.update(self.facade.as_mut().unwrap());
        self.universe.update(&delta, &self.context);

        result
    }

    fn builder() -> SimulationBuilder<U> {
        SimulationBuilder { universe: None }
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
    resolution: u32,
}

impl SimulationContext {
    fn new() -> SimulationContext {
        SimulationContext {
            pressed_keys: HashSet::new(),
            pressed_mouse_buttons: HashSet::new(),
            mouse: na::origin(),
            delta_mouse: na::zero(),
            resolution: 8,
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
                        }
                        ElementState::Released => {
                            self.pressed_keys
                                .remove_if(|tuple: &(u8, Option<VirtualKeyCode>)| {
                                    tuple.0 == character
                                });
                            if virtual_code.map_or(false, |virtual_code| {
                                virtual_code == VirtualKeyCode::Escape
                            }) {
                                return Err(event);
                            }
                        }
                    };
                },
                Event::MouseInput(state, button) => {
                    match state {
                        ElementState::Pressed => {
                            self.pressed_mouse_buttons.insert(button);
                        }
                        ElementState::Released => {
                            self.pressed_mouse_buttons.remove(&button);
                        }
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

                    window.set_cursor_position(center_x, center_y)
                        .expect("Could not reset the cursor position.");
                },
                Event::MouseWheel(mouse_scroll_delta, touch_phase) => {
                    let up: bool;
                    let down: bool;
                    match mouse_scroll_delta {
                        MouseScrollDelta::LineDelta(delta_x, delta_y) => {
                            up = delta_y > 0.0;
                            down = delta_y < 0.0;
                        },
                        MouseScrollDelta::PixelDelta(delta_x, delta_y) => {
                            up = delta_y > 0.0;
                            down = delta_y < 0.0;
                        },
                    }
                    if up && self.resolution > 1 {
                        self.resolution -= 1;
                    } else if down {
                        self.resolution += 1;
                    }
                },
                Event::Closed => {
                    return Err(event);
                },
                _ => (),
            }
        }
        Ok(())
    }
}

fn get_reflection_ratio_test(context: &TracingContext<Point3<f32>>) -> f32 {
    1.0
}

fn get_reflection_direction_test(context: &TracingContext<Point3<f32>>) -> Vector3<f32> {
    // R = 2*(V dot N)*N - V
    let normal = context.intersection_traceable.shape().get_normal_at(&context.intersection.location);
    -2.0 * na::dot(&context.intersection.direction, &normal) * normal + context.intersection.direction
}

fn get_surface_color_test(context: &TracingContext<Point3<f32>>) -> Rgba<u8> {
    Rgba {
        data: [255u8, 255u8, 0u8, 255u8],
    }
}

fn main() {
    let mut universe = Universe3D::new();

    {
        let mut entities = universe.entities_mut();
        entities.push(Box::new(Entity3Impl::new(
                Box::new(Sphere3::new(
                    Point3::new(2.0, 1.0, 0.0),
                    1.0
                )),
                Box::new(Vacuum::new()),
                Some(Box::new(PerlinSurface3::rand(&mut StdRng::new().expect("Could not create a random number generator."), 1.0, 1.0)))
            )));
        entities.push(Box::new(Entity3Impl::new(
                Box::new(Test3 {}),
                Box::new(Vacuum::new()),
                // Some(Box::new(PerlinSurface3::rand(&mut StdRng::new().expect("Could not create a random number generator."), 0.5, 2.0)))
                Some(Box::new(ComposableSurface {
                    reflection_ratio: get_reflection_ratio_test,
                    reflection_direction: get_reflection_direction_test,
                    surface_color: get_surface_color_test,
                }))
            )));
        entities.push(Box::new(Void::new_with_vacuum()));
    }

    {
        let mut intersectors = universe.intersectors_mut();
        intersectors.insert((Vacuum::id_static(), Test3::id_static()),
            universe::d3::entity::intersect_test);
        intersectors.insert((Vacuum::id_static(), VoidShape::id_static()),
            universe::d3::entity::intersect_void);
        intersectors.insert((Vacuum::id_static(), Sphere3::id_static()),
            universe::d3::entity::intersect_void);
        intersectors.insert((Vacuum::id_static(), Sphere3::id_static()),
            universe::d3::entity::intersect_sphere_in_vacuum);
    }

    let simulation = Simulation::builder()
        .universe(universe)
        .build();

    simulation.start();
}
