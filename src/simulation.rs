use std::collections::HashSet;
use std::time::Instant;
use std::time::Duration;
use std::marker::PhantomData;
use na;
use na::Point2;
use na::Vector2;
use glium::DisplayBuild;
use glium::backend::glutin_backend::GlutinFacade;
use glium::Surface as GliumSurface;
use glium::glutin::VirtualKeyCode;
use glium::glutin::MouseButton;
use glium::glutin::ElementState;
use glium::glutin::Event;
use glium::glutin::CursorState;
use glium::glutin::MouseScrollDelta;
use glium::glutin::WindowBuilder;
use glium::BlitTarget;
use glium::texture::Texture2d;
use glium::uniforms::MagnifySamplerFilter;
use universe::Environment;
use util::RemoveIf;
use util::CustomFloat;

pub struct Simulation<F: CustomFloat> {
    environment: Box<Environment<F>>,
    facade: Option<GlutinFacade>,
    start_instant: Option<Instant>,
    last_updated_instant: Option<Instant>,
    context: SimulationContext,
    float_precision: PhantomData<F>,
}

pub struct SimulationBuilder<F: CustomFloat> {
    environment: Option<Box<Environment<F>>>,
    float_precision: PhantomData<F>,
}

impl<F: CustomFloat> Simulation<F> {
    pub fn start(mut self) {
        let facade: GlutinFacade = WindowBuilder::new()
            .with_dimensions(1024, 768)
            .with_title(format!("euclider {}", crate_version!()))
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
            if let Err(_) = self.update() {
                break 'simulation;
            }

            self.render();
        }
    }

    fn render(&mut self) {
        let frame = self.facade.as_mut().unwrap().draw();
        let readable_facade = self.facade.as_ref().unwrap();
        let dimensions = frame.get_dimensions();
        let (width, height) = dimensions;
        let now = Instant::now();
        let time = now - self.start_instant.unwrap();

        let image = self.environment.render(dimensions, &time, &self.context);

        let texture = Texture2d::new(readable_facade, image).unwrap();
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

        self.environment.update(&delta, &self.context);

        result
    }

    pub fn builder() -> SimulationBuilder<F> {
        SimulationBuilder {
            environment: None,
            float_precision: PhantomData,
        }
    }
}

impl<F: CustomFloat> SimulationBuilder<F> {
    pub fn environment(mut self, environment: Box<Environment<F>>) -> SimulationBuilder<F> {
        self.environment = Some(environment);
        self
    }

    pub fn build(self) -> Simulation<F> {
        Simulation {
            environment: self.environment.unwrap(),
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
    pub pressed_keys: HashSet<(u8, Option<VirtualKeyCode>)>,
    pub pressed_mouse_buttons: HashSet<MouseButton>,
    pub mouse: Point2<i32>,
    pub delta_mouse: Vector2<i32>,
    pub resolution: u32,
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
                            let is_escape = |virtual_code| virtual_code == VirtualKeyCode::Escape;
                            if virtual_code.map_or(false, is_escape) {
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
                        MouseScrollDelta::LineDelta(delta_x, delta_y) |
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
