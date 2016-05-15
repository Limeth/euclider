use std::time::Duration;
use glium::Surface;

pub trait Universe {
    fn render<S: Surface>(&self, surface: &mut S, time: &Duration);
    fn update(&mut self, delta_time: &Duration);
}

pub struct Universe3D {

}

impl Universe3D {
    pub fn new() -> Universe3D {
        Universe3D {

        }
    }
}

impl Universe for Universe3D {
    fn render<S: Surface>(&self, surface: &mut S, time: &Duration) {

    }

    fn update(&mut self, delta_time: &Duration) {

    }
}
