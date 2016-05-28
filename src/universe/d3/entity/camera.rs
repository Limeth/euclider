extern crate nalgebra as na;
extern crate image;
extern crate std;

use std::collections::HashSet;
use std::time::Duration;
use self::na::*;
use glium::glutin::VirtualKeyCode;
use universe::entity::Entity;
use universe::entity::Locatable;
use universe::entity::Rotatable;
use universe::entity::Updatable;
use universe::entity::Traceable;
use universe::entity::Camera;
use universe::d3::entity::*;
use SimulationContext;

#[derive(Clone, Copy, PartialEq)]
pub struct Camera3Impl {
    location: Point3<f32>,
    rotation: Vector3<f32>,
    mouse_sensitivity: f32,
    speed: f32,
    fov: u8,
}

unsafe impl Sync for Camera3Impl {}

impl Camera3Impl {
    pub fn new() -> Camera3Impl {
        Camera3Impl {
            location: na::origin(),
            rotation: Vector3::new(1f32, 0f32, 0f32),
            mouse_sensitivity: 0.01,
            speed: 0.1,
            fov: 90,
        }
    }

    fn update_rotation(&mut self, context: &SimulationContext) {
        let delta_mouse_float: Vector2<f32> = Vector2::new(context.delta_mouse.x as f32,
                                                           context.delta_mouse.y as f32);

        if na::distance_squared(&na::origin(), delta_mouse_float.as_point()) <= 0f32 {
            return;
        }

        let direction = delta_mouse_float * self.mouse_sensitivity;
        let quaternion = UnitQuaternion::new_with_euler_angles(0f32, direction.y, direction.x);//, direction.y);
        self.rotation = quaternion.rotate(&self.rotation);
    }
}

impl Camera<Point3<f32>, Vector3<f32>> for Camera3Impl {
    #[allow(unused_variables)]
    fn get_ray_point(&self,
                     screen_x: i32,
                     screen_y: i32,
                     screen_width: i32,
                     screen_height: i32)
                     -> Point3<f32> {
        self.location
    }

    // TODO: Optimize - compute the step_yaw/step_pitch variables lazily and cache them
    fn get_ray_vector(&self,
                      screen_x: i32,
                      screen_y: i32,
                      screen_width: i32,
                      screen_height: i32)
                      -> Vector3<f32> {
        let rel_x = (screen_x - screen_width / 2) as f32 + (1 - screen_width % 2) as f32 / 2.0;
        let rel_y = (screen_y - screen_height / 2) as f32 + (1 - screen_height % 2) as f32 / 2.0;
        let screen_width = screen_width as f32;
        let screen_height = screen_height as f32;
        let fov_rad = std::f32::consts::PI * (self.fov as f32) / 180.0;

        let step_angle_partial = 2.0 * (fov_rad / 2.0).tan() /
                                 (screen_width * screen_width + screen_height * screen_height)
                                 .sqrt();
        let yaw = (step_angle_partial * rel_x).atan();
        let pitch = -(step_angle_partial * rel_y).atan();
        let quaternion = UnitQuaternion::new_with_euler_angles(0f32, pitch, yaw);
        // TODO remove this debug
        if screen_x == 0 && (screen_y == 0 || screen_y == screen_height as i32 - 1) {
                // println!("step_yaw: {}; step_pitch: {}", step_yaw, step_pitch);
                println!("yaw: {}; pitch: {}", yaw, pitch);
                let dir = quaternion.rotate(&self.rotation);
                println!("result_dir: <{}, {}, {}>", dir.x, dir.y, dir.z);
        }
        quaternion.rotate(&self.rotation)
    }
}

impl Entity<Point3<f32>, Vector3<f32>> for Camera3Impl {
    fn as_updatable_mut(&mut self) -> Option<&mut Updatable> {
        Some(self)
    }

    fn as_updatable(&self) -> Option<&Updatable> {
        Some(self)
    }

    fn as_traceable_mut(&mut self) -> Option<&mut Traceable<Point3<f32>, Vector3<f32>>> {
        None
    }

    fn as_traceable(&self) -> Option<&Traceable3> {
        None
    }
}

impl Updatable for Camera3Impl {
    fn update(&mut self, delta_time: &Duration, context: &SimulationContext) {
        self.update_rotation(context);

        let pressed_keys: &HashSet<(u8, Option<VirtualKeyCode>)> = context.pressed_keys();

        for pressed_key in pressed_keys {
            if pressed_key.1.is_none() {
                continue;
            }

            match pressed_key.1.unwrap() {
                VirtualKeyCode::W => {
                    self.location += self.rotation * self.speed;
                },
                VirtualKeyCode::S => {
                    self.location += -self.rotation * self.speed;
                },
                VirtualKeyCode::A => {
                    let quaternion = UnitQuaternion::new_with_euler_angles(0.0, std::f32::consts::PI / 2.0, 0.0);
                    let vector = quaternion.rotate(&self.rotation.clone());
                    self.location += vector * self.speed;
                },
                VirtualKeyCode::D => {
                    let quaternion = UnitQuaternion::new_with_euler_angles(0.0, -std::f32::consts::PI / 2.0, 0.0);
                    let vector = quaternion.rotate(&self.rotation.clone());
                    self.location += vector * self.speed;
                },
                _ => (),
            }
        }

        println!("location: {}\trotation: {}", self.location, self.rotation);
    }
}

impl Locatable<Point3<f32>> for Camera3Impl {
    fn location_mut(&mut self) -> &mut Point3<f32> {
        &mut self.location
    }

    fn location(&self) -> &Point3<f32> {
        &self.location
    }

    fn set_location(&mut self, location: Point3<f32>) {
        self.location = location;
    }
}

impl Rotatable<Vector3<f32>> for Camera3Impl {
    fn rotation_mut(&mut self) -> &mut Vector3<f32> {
        &mut self.rotation
    }

    fn rotation(&self) -> &Vector3<f32> {
        &self.rotation
    }

    fn set_rotation(&mut self, rotation: Vector3<f32>) {
        self.rotation = rotation;
    }
}
