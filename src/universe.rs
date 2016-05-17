extern crate nalgebra as na;

use std::time::Duration;
use self::na::*;
use glium::Surface;
use ::SimulationContext;

pub trait Universe {
    fn get_camera(&self) -> Camera;
    fn set_camera(&mut self, camera: Camera);

    fn render<S: Surface>(&self, surface: &mut S, time: &Duration, context: &SimulationContext) {
        let (width, height) = surface.get_dimensions();
        surface.clear_color(0.0, 0.0, 1.0, 1.0);
    }

    fn update(&mut self, delta_time: &Duration, context: &SimulationContext) {

    }
}

pub struct Universe3D {
    camera: Camera,
}

impl Universe3D {
    pub fn new() -> Universe3D {
        Universe3D {
            camera: Camera::new(),
        }
    }
}

impl Universe for Universe3D {
    fn get_camera(&self) -> Camera {
        self.camera
    }

    fn set_camera(&mut self, camera: Camera) {
        self.camera = camera;
    }
}

pub trait Locatable<P: NumPoint<i32>> {
    fn get_location(&self) -> P;
    fn set_location(&mut self, location: P);
}

pub trait Rotatable<P: NumVector<i32>> {
    fn get_rotation(&self) -> P;
    fn set_rotation(&mut self, location: P);
}

#[derive(Clone, Copy, Eq, PartialEq)]
pub struct Camera {
    location: Point3<i32>,
    rotation: Vector3<i32>,
}

impl Camera {
    fn new() -> Camera {
        Camera {
            location: na::origin(),
            rotation: na::zero(),
        }
    }
}

impl Locatable<Point3<i32>> for Camera {
    fn get_location(&self) -> Point3<i32> {
        self.location
    }

    fn set_location(&mut self, location: Point3<i32>) {
        self.location = location;
    }
}

impl Rotatable<Vector3<i32>> for Camera {
    fn get_rotation(&self) -> Vector3<i32> {
        self.rotation
    }

    fn set_rotation(&mut self, rotation: Vector3<i32>) {
        self.rotation = rotation;
    }
}
