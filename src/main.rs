#![feature(reflect_marker)]
#![feature(custom_attribute)]
#![feature(const_fn)]
#![feature(zero_one)]

extern crate core;
extern crate nalgebra as na;
extern crate scoped_threadpool;
extern crate image;
extern crate noise;
extern crate rand;
extern crate palette;
extern crate glium;
extern crate num;
mod universe;
pub mod util;

use std::collections::HashSet;
use std::time::Instant;
use std::time::Duration;
use std::marker::PhantomData;
use rand::StdRng;
use na::Cast;
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
use palette::Alpha;
use palette::Rgb;
use palette::Rgba;
use palette::Hsv;
use palette::RgbHue;
use universe::Universe;
use universe::entity::*;
use universe::d3::Universe3D;
use universe::d3::entity::*;
use util::RemoveIf;
use util::CustomFloat;
use util::HasId;
use na::BaseFloat;
use num::traits::NumCast;
use num::Zero;

pub struct Simulation<F: CustomFloat, U: Universe<F>> {
    universe: Box<U>,
    facade: Option<GlutinFacade>,
    start_instant: Option<Instant>,
    last_updated_instant: Option<Instant>,
    context: SimulationContext,
    float_precision: PhantomData<F>,
}

struct SimulationBuilder<F: CustomFloat, U: Universe<F>> {
    universe: Option<Box<U>>,
    float_precision: PhantomData<F>,
}

impl<F: CustomFloat, U: Universe<F>> Simulation<F, U> {
    fn start(mut self) {
        let facade: GlutinFacade = glium::glutin::WindowBuilder::new()
            .with_dimensions(1024, 768)
            .with_title(format!("Hello world"))
            .build_glium()
            .unwrap();

        facade.get_window()
            .unwrap()
            .set_cursor_state(CursorState::Hide)
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

    fn builder() -> SimulationBuilder<F, U> {
        SimulationBuilder {
            universe: None,
            float_precision: PhantomData,
        }
    }
}

impl<F: CustomFloat, U: Universe<F>> SimulationBuilder<F, U> {
    fn universe(mut self, universe: U) -> SimulationBuilder<F, U> {
        self.universe = Some(Box::new(universe));
        self
    }

    fn build(self) -> Simulation<F, U> {
        Simulation {
            universe: self.universe.unwrap(),
            facade: None,
            start_instant: None,
            last_updated_instant: None,
            context: SimulationContext::new(),
            float_precision: PhantomData,
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

    #[allow(unused_variables)]
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
                }
                Event::MouseInput(state, button) => {
                    match state {
                        ElementState::Pressed => {
                            self.pressed_mouse_buttons.insert(button);
                        }
                        ElementState::Released => {
                            self.pressed_mouse_buttons.remove(&button);
                        }
                    }
                }
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
                }
                Event::MouseWheel(mouse_scroll_delta, touch_phase) => {
                    let up: bool;
                    let down: bool;
                    match mouse_scroll_delta {
                        MouseScrollDelta::LineDelta(delta_x, delta_y) => {
                            up = delta_y > 0.0;
                            down = delta_y < 0.0;
                        }
                        MouseScrollDelta::PixelDelta(delta_x, delta_y) => {
                            up = delta_y > 0.0;
                            down = delta_y < 0.0;
                        }
                    }
                    if up && self.resolution > 1 {
                        self.resolution -= 1;
                    } else if down {
                        self.resolution += 1;
                    }
                }
                Event::Closed => {
                    return Err(event);
                }
                _ => (),
            }
        }
        Ok(())
    }
}

#[allow(unused_variables)]
fn get_reflection_ratio_test<F: CustomFloat>(context: &TracingContext<F, Point3<F>, Vector3<F>>) -> F {
    Cast::from(0.5)
}

fn get_reflection_direction_test<F: CustomFloat>(context: &TracingContext<F, Point3<F>, Vector3<F>>)
                                                 -> Vector3<F> {
    // R = 2*(V dot N)*N - V
    let mut normal = context.intersection.normal;

    if na::angle_between(&context.intersection.direction, &normal) > BaseFloat::frac_pi_2() {
        normal = -normal;
    }

    normal * <F as NumCast>::from(-2.0).unwrap() *
    na::dot(&context.intersection.direction, &normal) + context.intersection.direction
}

fn get_surface_color_test<F: CustomFloat>(context: &TracingContext<F, Point3<F>, Vector3<F>>) -> Rgba<F> {
    let mut normal = context.intersection.normal;

    if na::angle_between(&context.intersection.direction, &normal) > BaseFloat::frac_pi_2() {
        normal = -normal;
    }

    let angle: F = na::angle_between(&normal, &universe::d3::entity::AXIS_Z());

    Alpha {
        color: Rgb::from(
                    Hsv::new(
                        RgbHue::from(<F as Zero>::zero()),
                        <F as Zero>::zero(),
                        <F as NumCast>::from(angle / <F as BaseFloat>::pi()).unwrap()
                    )
               ),
        alpha: Cast::from(0.5),
    }
}

#[allow(unused_variables)]
fn transition_vacuum_vacuum<F: CustomFloat>(from: &Material<F, Point3<F>, Vector3<F>>,
                                            to: &Material<F, Point3<F>, Vector3<F>>,
                                            context: &TracingContext<F, Point3<F>, Vector3<F>>)
                                            -> Option<Rgba<F>> {
    let trace = context.trace;
    trace(context.time,
          context.intersection_traceable,
          &context.intersection.location,
          &context.intersection.direction)
}

fn main() {
    run::<f64>();
}

fn run<F: CustomFloat>() {
    let mut universe: Universe3D<F> = Universe3D::new();

    {
        let mut entities = universe.entities_mut();
        entities.push(Box::new(Entity3Impl::new(
                Box::new(Sphere3::<F>::new(
                    Point3::new(Cast::from(2.0),
                                Cast::from(1.0),
                                Cast::from(0.0)),
                    Cast::from(1.0)
                )),
                Box::new(Vacuum::new()),
                // Some(Box::new(PerlinSurface3::rand(&mut StdRng::new().expect("Could not create a random number generator."), 1.0, 1.0)))
                Some(Box::new(ComposableSurface {
                    reflection_ratio: get_reflection_ratio_test,
                    reflection_direction: get_reflection_direction_test,
                    surface_color: get_surface_color_test,
                }))
            )));
        entities.push(Box::new(Entity3Impl::new(
                Box::new(Sphere3::<F>::new(
                        Point3::new(Cast::from(2.0),
                                    Cast::from(-1.5),
                                    Cast::from(0.0)),
                        Cast::from(1.25)
                        )),
                Box::new(Vacuum::new()),
                Some(Box::new(PerlinSurface3::rand(
                            &mut StdRng::new().expect("Could not create a random number generator."),
                            Cast::from(2.0),
                            Cast::from(1.0))))
                // Some(Box::new(ComposableSurface {
                //     reflection_ratio: get_reflection_ratio_test,
                //     reflection_direction: get_reflection_direction_test,
                //     surface_color: get_surface_color_test,
                // }))
            )));
        entities.push(Box::new(Entity3Impl::new(
                Box::new(Sphere3::<F>::new(
                        Point3::new(Cast::from(2.0),
                                    Cast::from(10.5), // FIXME: The Y-coordinate seems to be flipped.
                                    Cast::from(0.0)),
                        Cast::from(2.25)
                        )),
                Box::new(Vacuum::new()),
                Some(Box::new(PerlinSurface3::rand(
                            &mut StdRng::new().expect("Could not create a random number generator."),
                            Cast::from(2.0),
                            Cast::from(1.0))))
                // Some(Box::new(ComposableSurface {
                //     reflection_ratio: get_reflection_ratio_test,
                //     reflection_direction: get_reflection_direction_test,
                //     surface_color: get_surface_color_test,
                // }))
            )));
        entities.push(Box::new(Entity3Impl::new(
                    Box::new(ComposableShape::new(
                        HalfSpace3::from_point(
                            // Plane3::from_equation(Cast::from(1.0),
                            //                       Cast::from(0.5),
                            //                       Cast::from(0.0),
                            //                       Cast::from(-1.0)),
                            Plane3::new(
                                &Point3::new(Cast::from(0.0),
                                Cast::from(3.0),
                                Cast::from(3.0)),
                                &Vector3::new(Cast::from(0.0),
                                Cast::from(3.0),
                                Cast::from(1.0)),
                                &Vector3::new(Cast::from(1.0),
                                Cast::from(0.0),
                                Cast::from(0.0)),
                            ),
                            &Point3::new(Cast::from(0.0),
                            Cast::from(0.0),
                            Cast::from(-100.0))
                        ),
                        HalfSpace3::from_point(
                            // Plane3::from_equation(Cast::from(1.0),
                            //                       Cast::from(0.5),
                            //                       Cast::from(0.0),
                            //                       Cast::from(-1.0)),
                            Plane3::new(
                                &Point3::new(Cast::from(0.0),
                                Cast::from(3.0),
                                Cast::from(3.0)),
                                &Vector3::new(Cast::from(0.0),
                                Cast::from(3.0),
                                Cast::from(-1.0)),
                                &Vector3::new(Cast::from(1.0),
                                Cast::from(0.0),
                                Cast::from(0.0)),
                            ),
                            &Point3::new(Cast::from(0.0),
                            Cast::from(0.0),
                            Cast::from(100.0))
                        ),
                        SetOperation::Intersection
                        )),
                        Box::new(Vacuum::new()),
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
        intersectors.insert((Vacuum::id_static(), VoidShape::id_static()),
                            universe::d3::entity::intersect_void);
        intersectors.insert((Vacuum::id_static(), Sphere3::<F>::id_static()),
                            universe::d3::entity::intersect_void);
        intersectors.insert((Vacuum::id_static(), Sphere3::<F>::id_static()),
                            universe::d3::entity::intersect_sphere_in_vacuum);
        intersectors.insert((Vacuum::id_static(), Plane3::<F>::id_static()),
                            universe::d3::entity::intersect_plane_in_vacuum);
        intersectors.insert((Vacuum::id_static(), HalfSpace3::<F>::id_static()),
                            universe::d3::entity::intersect_halfspace_in_vacuum);
        intersectors.insert((Vacuum::id_static(), ComposableShape::<F, Point3<F>, Vector3<F>>::id_static()),
                            ComposableShape::<F, Point3<F>, Vector3<F>>::intersect_in_vacuum);
    }

    {
        let mut transitions = universe.transitions_mut();
        transitions.insert((Vacuum::id_static(), Vacuum::id_static()),
                           transition_vacuum_vacuum);
    }

    let simulation = Simulation::builder()
        .universe(universe)
        .build();

    simulation.start();
}
