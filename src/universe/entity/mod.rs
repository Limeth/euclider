extern crate nalgebra as na;
extern crate image;

use std::time::Duration;
use self::na::NumPoint;
use self::na::NumVector;
use self::image::Rgba;
use SimulationContext;

pub trait Entity<P: NumPoint<f32>, V: NumVector<f32>> {
    fn as_updatable_mut(&mut self) -> Option<&mut Updatable>;
    fn as_updatable(&self) -> Option<&Updatable>;
    fn as_traceable_mut(&mut self) -> Option<&mut Traceable<P, V>>;
    fn as_traceable(&self) -> Option<&Traceable<P, V>>;
}

pub trait Camera<P: NumPoint<f32>, V: NumVector<f32>>: Entity<P, V> {
    fn get_ray_point(&self, screen_x: i32, screen_y: i32, screen_width: i32, screen_height: i32) -> P;
    fn get_ray_vector(&self, screen_x: i32, screen_y: i32, screen_width: i32, screen_height: i32) -> V;
}

pub trait Shape<P: NumPoint<f32>, V: NumVector<f32>> {
    fn get_normal_at(&self, point: &P) -> V;
    fn is_point_inside(&self, point: &P) -> bool;
}

pub trait Material<P: NumPoint<f32>, V: NumVector<f32>> {

}

pub trait Surface<P: NumPoint<f32>, V: NumVector<f32>> {

}

pub trait Updatable {
    fn update(&mut self, delta_time: &Duration, context: &SimulationContext);
}

pub trait Traceable<P: NumPoint<f32>, V: NumVector<f32>> {
    fn trace(&self) -> Rgba<u8>;
    fn shape(&self) -> &Shape<P, V>;
    fn material(&self) -> Option<&Material<P, V>>;
    fn surface(&self) -> Option<&Surface<P, V>>;
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

pub struct VoidShape {}

impl VoidShape {
    pub fn new() -> VoidShape {
        VoidShape {}
    }
}

impl<P: NumPoint<f32>, V: NumVector<f32>> Shape<P, V> for VoidShape {
    fn get_normal_at(&self, point: &P) -> V {
        na::zero()
    }

    fn is_point_inside(&self, point: &P) -> bool {
        true
    }
}

pub struct Void {
    shape: Box<VoidShape>,
}

impl Void {
    pub fn new() -> Void {
        Void {
            shape: Box::new(VoidShape::new()),
        }
    }
}

impl<P: NumPoint<f32>, V: NumVector<f32>> Entity<P, V> for Void {
    fn as_updatable_mut(&mut self) -> Option<&mut Updatable> {
        None
    }

    fn as_updatable(&self) -> Option<&Updatable> {
        None
    }

    fn as_traceable_mut(&mut self) -> Option<&mut Traceable<P, V>> {
        Some(self)
    }

    fn as_traceable(&self) -> Option<&Traceable<P, V>> {
        Some(self)
    }
}

impl<P: NumPoint<f32>, V: NumVector<f32>> Traceable<P, V> for Void {
    fn trace(&self) -> Rgba<u8> {
        // TODO
        Rgba {
            data: [0u8, 0u8, 255u8, 255u8],
        }
    }

    fn shape(&self) -> &Shape<P, V> {
        self.shape.as_ref()
    }

    fn material(&self) -> Option<&Material<P, V>> {
        None
    }

    fn surface(&self) -> Option<&Surface<P, V>> {
        None
    }
}
