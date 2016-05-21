extern crate nalgebra as na;
extern crate image;
pub mod camera;

use std::time::Duration;
use self::na::NumPoint;
use self::na::NumVector;
use self::image::Rgba;
use SimulationContext;

pub trait Entity<P: NumPoint<f32>, V: NumVector<f32>> {
    fn as_updatable(&mut self) -> Option<&mut Updatable>;
    fn as_traceable(&mut self) -> Option<&mut Traceable<P>>;
}

pub trait Shape {

}

pub trait Material {

}

pub trait Surface {

}

pub trait Updatable {
    fn update(&mut self, delta_time: &Duration, context: &SimulationContext);
}

pub trait Traceable<P: NumPoint<f32>> {
    fn trace(&self) -> Rgba<u8>;
    fn is_point_inside(&self, point: P);
    fn shape(&self) -> &Shape;
    fn material(&self) -> &Option<Box<&Material>>;
    fn surface(&self) -> Option<Box<&Surface>>;
}

pub trait Locatable<P: NumPoint<f32>> {
    fn location_mut(&mut self) -> &mut P;
    fn location(&self) -> &P;
    fn set_location(&mut self, location: P);
}

pub trait Rotatable<P: NumVector<f32>> {
    fn rotation_mut(&mut self) -> &mut P;
    fn rotation(&self) -> &P;
    fn set_rotation(&mut self, location: P);
}
