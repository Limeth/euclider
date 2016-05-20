extern crate nalgebra as na;
extern crate palette;
extern crate glium;
pub mod d3;
mod camera;

use std::time::Duration;
use self::na::*;
use glium::Surface;
use self::palette::Rgb;
use self::palette::Rgba;
use ::SimulationContext;
use universe::camera::Camera;

pub trait Universe {
    fn camera_mut(&mut self) -> &mut Camera;
    fn camera(&self) -> &Camera;
    fn set_camera(&mut self, camera: &Camera);
    fn entities_mut(&mut self) -> &mut Vec<Box<Entity>>;
    fn entities(&self) -> &Vec<Box<Entity>>;
    fn set_entities(&mut self, entities: Vec<Box<Entity>>);

    fn render<S: Surface>(&self, surface: &mut S, time: &Duration, context: &SimulationContext) {
        let (width, height) = surface.get_dimensions();
        surface.clear_color(0.0, 0.0, 1.0, 1.0);

        // TODO: Render stuff
    }

    fn update(&mut self, delta_time: &Duration, context: &SimulationContext) {
        self.camera_mut().update(delta_time, context);
    }
}

pub trait Entity {
    fn as_updatable(&mut self) -> Option<&mut Updatable>;
    // fn as_drawable<T: Drawable>(&mut self) -> &mut T;
    fn as_traceable(&mut self) -> Option<&mut Traceable>;
}

pub trait Updatable {
    fn update(&mut self, delta_time: &Duration, context: &SimulationContext);
}

// pub trait Drawable {
//     fn render<S: Surface>(&self, surface: &mut S, time: &Duration, context: &SimulationContext);
// }

pub trait Traceable {
    fn trace(&self) -> Rgba;
}

pub trait Locatable<P: NumPoint<f32>> {
    fn get_location(&self) -> P;
    fn set_location(&mut self, location: P);
}

pub trait Rotatable<P: NumVector<f32>> {
    fn get_rotation(&self) -> P;
    fn set_rotation(&mut self, location: P);
}
