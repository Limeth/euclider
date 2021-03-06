use ::F;
use std::collections::HashSet;
use std::time::Instant;
use std::time::Duration;
use num::One;
use na;
use na::Cast;
use na::Point2;
use na::Vector2;
use na::BaseFloat;
use glium::Surface as GliumSurface;
use glium::BlitTarget;
use glium::backend::glutin::Display;
use glium::glutin::ContextBuilder;
use glium::glutin::Event;
use glium::glutin::dpi::LogicalPosition;
use glium::glutin::EventsLoop;
use glium::glutin::KeyboardInput;
use glium::glutin::VirtualKeyCode;
use glium::glutin::MouseButton;
use glium::glutin::ElementState;
use glium::glutin::WindowEvent;
use glium::glutin::MouseScrollDelta;
use glium::glutin::MouseCursor;
use glium::glutin::WindowBuilder;
use glium::texture::Texture2d;
use glium::uniforms::MagnifySamplerFilter;
use universe::Environment;
use util::CustomFloat;

pub struct Simulation {
    events_loop: Option<EventsLoop>,
    debug: bool,
    threads: u32,
    environment: Box<Environment>,
    display: Option<Display>,
    start_instant: Option<Instant>,
    last_updated_instant: Option<Instant>,
    context: SimulationContext,
}

pub struct SimulationBuilder {
    environment: Option<Box<Environment>>,
    threads: Option<u32>,
    debug: bool,
}

impl Simulation {
    pub fn start(mut self) {
        let window_builder = WindowBuilder::new()
            .with_dimensions((1024, 768).into())
            .with_title(format!("euclider {}", crate_version!()));
        let events_loop = EventsLoop::new();
        let display: Display = Display::new(
            window_builder,
            ContextBuilder::new(),
            &events_loop,
        ).unwrap();

        (*display.gl_window())
            .hide_cursor(true);

        self.display = Some(display);
        self.events_loop = Some(events_loop);
        self.start_instant = Some(Instant::now());

        self.render();

        'simulation: loop {
            if self.update().is_err() {
                break 'simulation;
            }

            self.render();
        }
    }

    fn render(&mut self) {
        let frame = self.display.as_mut().unwrap().draw();
        let readable_display = self.display.as_ref().unwrap();
        let dimensions = frame.get_dimensions();
        let (width, height) = dimensions;
        let now = Instant::now();
        let time = now - self.start_instant.unwrap();

        let image = self.environment.render(dimensions, &time, self.threads, &self.context);

        let texture = Texture2d::new(readable_display, image).unwrap();
        let image_surface = texture.as_surface();
        let blit_target = BlitTarget {
            left: 0,
            bottom: 0,
            width: width as i32,
            height: height as i32,
        };
        image_surface.blit_whole_color_to(&frame, &blit_target, MagnifySamplerFilter::Nearest);

        if let Err(error) = frame.finish() {
            panic!("An error occured while swapping the OpenGL buffers: {:?}",
                   error)
        }
    }

    fn update(&mut self) -> Result<(), WindowEvent> {
        let delta: Duration;
        let now = Instant::now();

        if self.last_updated_instant.is_none() {
            delta = now - self.start_instant.unwrap();
        } else {
            delta = now - self.last_updated_instant.unwrap();
        }

        if self.context.debugging {
            let delta_millis: F = 1.0 / ((delta * 1000).as_secs() as f64 / 1000.0) as F;

            println!("FPS: {}", delta_millis);
        }

        self.last_updated_instant = Some(now);
        let result = self.context.update(&mut self.events_loop, self.display.as_ref().unwrap(), self.debug);

        self.environment.update(&delta, &self.context);

        result
    }

    pub fn builder() -> SimulationBuilder {
        SimulationBuilder {
            environment: None,
            threads: None,
            debug: false,
        }
    }
}

impl SimulationBuilder {
    pub fn environment(mut self, environment: Box<Environment>) -> Self {
        self.environment = Some(environment);
        self
    }

    pub fn threads(mut self, threads: u32) -> Self {
        self.threads = Some(threads);
        self
    }

    pub fn debug(mut self, debug: bool) -> Self {
        self.debug = debug;
        self
    }

    pub fn build(self) -> Simulation {
        Simulation {
            events_loop: None,
            debug: self.debug,
            threads: self.threads.expect("Specify the number of threads before building the simulation."),
            environment: self.environment.expect("Specify the environment before bulding the simulation."),
            display: None,
            start_instant: None,
            last_updated_instant: None,
            context: SimulationContext::new(),
        }
    }
}

pub struct SimulationContext {
    pub pressed_keys: HashSet<VirtualKeyCode>,
    pub pressed_mouse_buttons: HashSet<MouseButton>,
    pub mouse: Point2<f64>,
    pub delta_mouse: Vector2<f64>,
    pub resolution: u32,
    pub debugging: bool,
}

impl SimulationContext {
    fn new() -> SimulationContext {
        SimulationContext {
            pressed_keys: HashSet::new(),
            pressed_mouse_buttons: HashSet::new(),
            mouse: na::origin(),
            delta_mouse: na::zero(),
            resolution: 8,
            debugging: false,
        }
    }

    pub fn pressed_keys(&self) -> &HashSet<VirtualKeyCode> {
        &self.pressed_keys
    }

    pub fn pressed_mouse_buttons(&self) -> &HashSet<MouseButton> {
        &self.pressed_mouse_buttons
    }

    pub fn delta_mouse(&self) -> Vector2<f64> {
        self.delta_mouse
    }

    fn reset_delta_mouse(&mut self) {
        self.delta_mouse = na::zero();
    }

    #[allow(unused_variables)]
    pub fn update(&mut self, events_loop: &mut Option<EventsLoop>, display: &Display, debug: bool) -> Result<(), WindowEvent> {
        if events_loop.is_none() {
            return Ok(());
        }

        let mut return_value: Result<(), WindowEvent> = Ok(());

        self.reset_delta_mouse();

        // TODO: Consider using `run_forever`
        events_loop.as_mut().unwrap().poll_events(|outer_event| {
            if let Event::WindowEvent { event, .. } = outer_event {
                match event {
                    WindowEvent::KeyboardInput {
                        input: KeyboardInput {
                            state,
                            virtual_keycode: Some(virtual_code),
                            ..
                        },
                        ..
                    } => {
                        match state {
                            ElementState::Pressed => {
                                self.pressed_keys.insert(virtual_code);

                                if debug && virtual_code == VirtualKeyCode::LAlt {
                                    self.debugging = true;
                                }
                            }
                            ElementState::Released => {
                                self.pressed_keys.remove(&virtual_code);

                                match virtual_code {
                                    VirtualKeyCode::Escape => {
                                        return_value = Err(event);
                                        return;
                                    },
                                    VirtualKeyCode::LAlt => {
                                        if debug {
                                            self.debugging = false;
                                        }
                                    },
                                    _ => ()
                                }

                            }
                        };
                    }
                    WindowEvent::MouseInput {
                        state,
                        button,
                        ..
                    } => {
                        match state {
                            ElementState::Pressed => {
                                self.pressed_mouse_buttons.insert(button);
                            }
                            ElementState::Released => {
                                self.pressed_mouse_buttons.remove(&button);
                            }
                        }
                    }
                    WindowEvent::CursorMoved {
                        position: LogicalPosition { x, y },
                        ..
                    } => {
                        let window = display.gl_window();
                        let window_size = window.get_inner_size().unwrap();
                        let center_x = window_size.width / 2.0;
                        let center_y = window_size.height / 2.0;

                        self.delta_mouse.x = self.mouse.x - x;
                        self.delta_mouse.y = self.mouse.y - y;
                        self.mouse.x = x;
                        self.mouse.y = y;

                        window.set_cursor_position((center_x, center_y).into())
                            .expect("Could not reset the cursor position.");
                    }
                    WindowEvent::MouseWheel {
                        delta: mouse_scroll_delta,
                        phase: touch_phase,
                        ..
                    } => {
                        let up: bool;
                        let down: bool;
                        match mouse_scroll_delta {
                            MouseScrollDelta::LineDelta(delta_x, delta_y) => {
                                up = delta_y > 0.0;
                                down = delta_y < 0.0;
                            },
                            MouseScrollDelta::PixelDelta(LogicalPosition { x: delta_x, y: delta_y }) => {
                                up = delta_y > 0.0;
                                down = delta_y < 0.0;
                            },
                        }
                        if up && self.resolution > 1 {
                            self.resolution -= 1;
                        } else if down {
                            self.resolution += 1;
                        }
                    }
                    WindowEvent::CloseRequested => {
                        return_value = Err(event);
                        return;
                    }
                    _ => (),
                }
            }
        });

        Ok(())
    }
}
