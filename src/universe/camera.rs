extern crate nalgebra as na;
extern crate image;

use std::time::Duration;
use self::na::*;
use self::image::Rgba;
use universe::Entity;
use universe::Locatable;
use universe::Rotatable;
use universe::Updatable;
use universe::Traceable;
use SimulationContext;

#[derive(Clone, Copy, PartialEq)]
pub struct Camera {
    location: Point3<f32>,
    rotation: Vector3<f32>,
}

impl Camera {
    pub fn new() -> Camera {
        Camera {
            location: na::origin(),
            rotation: Vector3::new(1f32, 0f32, 0f32),
        }
    }
}

impl Entity for Camera {
    fn as_updatable(&mut self) -> Option<&mut Updatable> {
        Some(self)
    }

    fn as_traceable(&mut self) -> Option<&mut Traceable> {
        Some(self)
    }
}

impl Updatable for Camera {
    fn update(&mut self, delta_time: &Duration, context: &SimulationContext) {
        let delta_mouse_float: Vector2<f32> = Vector2::new(context.delta_mouse.x as f32, context.delta_mouse.y as f32);

        if na::distance_squared(&na::origin(), delta_mouse_float.as_point()) <= 0f32 {
            return;
        }

        let mouse_sensitivity = 0.01;
        let direction = delta_mouse_float * mouse_sensitivity;
        let quaternion = UnitQuaternion::new_with_euler_angles(0f32, direction.x, direction.y);
        self.rotation = quaternion.rotate(&self.rotation);
    }
}

/// TODO: Remove this, because it's not really needed. Just for testing.
impl Traceable for Camera {
    fn trace(&self) -> Rgba<u8> {
        Rgba {
            data: [0u8, 255u8, 0u8, 255u8]
        }
    }
}

impl Locatable<Point3<f32>> for Camera {
    fn get_location(&self) -> Point3<f32> {
        self.location
    }

    fn set_location(&mut self, location: Point3<f32>) {
        self.location = location;
    }
}

impl Rotatable<Vector3<f32>> for Camera {
    fn get_rotation(&self) -> Vector3<f32> {
        self.rotation
    }

    fn set_rotation(&mut self, rotation: Vector3<f32>) {
        self.rotation = rotation;
    }
}
