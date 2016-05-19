extern crate nalgebra as na;

use std::time::Duration;
use self::na::*;
use universe::Locatable;
use universe::Rotatable;
use universe::Updatable;
use SimulationContext;

#[derive(Clone, Copy, Eq, PartialEq)]
pub struct Camera {
    location: Point3<i32>,
    rotation: Vector3<i32>,
}

impl Camera {
    pub fn new() -> Camera {
        Camera {
            location: na::origin(),
            rotation: na::zero(),
        }
    }
}

impl Updatable for Camera {
    fn update(&mut self, delta_time: &Duration, context: &SimulationContext) {
        println!("delta_mouse: {:?}", context.delta_mouse);
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
