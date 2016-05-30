use std;
use std::collections::HashSet;
use std::time::Duration;
use na;
use na::*;
use glium::glutin::VirtualKeyCode;
use universe::entity::Entity;
use universe::entity::Locatable;
use universe::entity::Rotatable;
use universe::entity::Updatable;
use universe::entity::Traceable;
use universe::entity::Camera;
use universe::d3::entity::*;
use SimulationContext;

const AXIS_Z: Vector3<f32> = Vector3 {
    x: 0.0,
    y: 0.0,
    z: 1.0,
};

#[derive(Clone, Copy, PartialEq)]
pub struct Camera3Impl {
    location: Point3<f32>,
    forward: Vector3<f32>,
    up: Vector3<f32>,
    mouse_sensitivity: f32,
    speed: f32,
    fov: u8,
}

unsafe impl Sync for Camera3Impl {}

impl Camera3Impl {
    pub fn new() -> Camera3Impl {
        Camera3Impl {
            location: na::origin(),
            forward: Vector3::new(1.0, 0.0, 0.0),
            up: AXIS_Z,
            mouse_sensitivity: 0.01,
            speed: 10.0,
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

        self.rotate_x(direction.x);
        self.rotate_y(-direction.y);
    }

    fn rotate_x_static(forward: &mut Vector3<f32>, up: &mut Vector3<f32>, angle: f32) {
        let axis_h = na::cross(forward, &AXIS_Z).normalize();
        let quaternion = UnitQuaternion::new(AXIS_Z * angle);
        *forward = quaternion.rotate(forward).normalize();
        *up = na::cross(&axis_h, forward).normalize();
    }

    fn rotate_y_static(forward: &mut Vector3<f32>, up: &mut Vector3<f32>, angle: f32, snap: bool) {
        let axis_h = na::cross(forward, &AXIS_Z).normalize();

        if snap && false {
            let result_angle = (na::dot(&forward.clone(), &AXIS_Z)
                                / (na::distance(&na::origin(), &forward.to_point())
                                   * na::distance(&na::origin(), &AXIS_Z.to_point()) as f32)).acos();

            println!("dot: {}", na::dot(&forward.clone(), &AXIS_Z).abs());
            println!("{} < {}", result_angle, angle);

            if result_angle < angle {
                // angle = result_angle;
                *forward = AXIS_Z;
                *up = na::cross(&axis_h, forward).normalize();
                return;
            } else if result_angle - std::f32::consts::PI > angle {
                // angle = result_angle - std::f32::consts::PI;
                *forward = -AXIS_Z;
                *up = na::cross(&axis_h, forward).normalize();
                return;
            }
        }

        let quaternion = UnitQuaternion::new(axis_h * angle);
        *forward = quaternion.rotate(forward).normalize();
        *up = na::cross(&axis_h, forward).normalize();
    }

    fn rotate_x(&mut self, angle: f32) {
        Camera3Impl::rotate_x_static(&mut self.forward, &mut self.up, angle);
    }

    fn rotate_y(&mut self, angle: f32) {
        Camera3Impl::rotate_y_static(&mut self.forward, &mut self.up, angle, true);
    }

    fn get_right(&self) -> Vector3<f32> {
        na::cross(&self.up, &self.forward).normalize()
    }

    fn get_left(&self) -> Vector3<f32> {
        na::cross(&self.forward, &self.up).normalize()
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
        let right = self.get_right();
        let fov_rad = std::f32::consts::PI * (self.fov as f32) / 180.0;
        let distance_from_screen_center = (screen_width * screen_width + screen_height * screen_height).sqrt()
            / (2.0 * (fov_rad / 2.0).tan());
        let screen_center_point_3d = self.location + self.forward * distance_from_screen_center;
        let screen_point_3d = screen_center_point_3d + (self.up * rel_y) + (right * rel_x);

        (screen_point_3d - self.location).normalize()
    }
}

impl Entity<Point3<f32>, Vector3<f32>> for Camera3Impl {
    fn as_updatable_mut(&mut self) -> Option<&mut Updatable<Point3<f32>, Vector3<f32>>> {
        Some(self)
    }

    fn as_updatable(&self) -> Option<&Updatable3> {
        Some(self)
    }

    fn as_traceable_mut(&mut self) -> Option<&mut Traceable<Point3<f32>, Vector3<f32>>> {
        None
    }

    fn as_traceable(&self) -> Option<&Traceable3> {
        None
    }
}

impl Updatable<Point3<f32>, Vector3<f32>> for Camera3Impl {
    fn update(&mut self, delta_time: &Duration, context: &SimulationContext) {
        self.update_rotation(context);

        let pressed_keys: &HashSet<(u8, Option<VirtualKeyCode>)> = context.pressed_keys();
        let delta_millis = (delta_time.clone() * 1000u32).as_secs() as f32 / 1000.0;

        for pressed_key in pressed_keys {
            if pressed_key.1.is_none() {
                continue;
            }

            match pressed_key.1.unwrap() {
                VirtualKeyCode::W => {
                    self.location += self.forward * self.speed * delta_millis;
                },
                VirtualKeyCode::S => {
                    self.location += -self.forward * self.speed * delta_millis;
                },
                VirtualKeyCode::A => {
                    self.location += self.get_left() * self.speed * delta_millis;
                },
                VirtualKeyCode::D => {
                    self.location += self.get_right() * self.speed * delta_millis;
                },
                VirtualKeyCode::LControl => {
                    self.location += -AXIS_Z * self.speed * delta_millis;
                },
                VirtualKeyCode::LShift => {
                    self.location += AXIS_Z * self.speed * delta_millis;
                },
                _ => (),
            }
        }
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
        &mut self.forward
    }

    fn rotation(&self) -> &Vector3<f32> {
        &self.forward
    }

    fn set_rotation(&mut self, rotation: Vector3<f32>) {
        self.forward = rotation;
    }
}
