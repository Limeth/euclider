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

#[derive(Clone, Copy, PartialEq)]
pub struct Camera3Impl<F: BaseFloat> {
    location: Point3<F>,
    forward: Vector3<F>,
    up: Vector3<F>,
    mouse_sensitivity: F,
    speed: F,
    fov: u8,
}

unsafe impl<F: BaseFloat> Sync for Camera3Impl<F> {}

impl<F: BaseFloat> Camera3Impl<F> {
    pub fn new() -> Camera3Impl<F> {
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
        let delta_mouse_float: Vector2<F> = Vector2::new(context.delta_mouse.x as F,
                                                           context.delta_mouse.y as F);

        if na::distance_squared(&na::origin(), delta_mouse_float.as_point()) <= 0.0 {
            return;
        }

        let direction = delta_mouse_float * self.mouse_sensitivity;

        self.rotate_x(direction.x);
        self.rotate_y(-direction.y);
    }

    fn rotate_x_static(forward: &mut Vector3<F>, up: &mut Vector3<F>, angle: F) {
        let axis_h = na::cross(forward, &AXIS_Z).normalize();
        let quaternion = UnitQuaternion::new(AXIS_Z * angle);
        *forward = quaternion.rotate(forward).normalize();
        *up = na::cross(&axis_h, forward).normalize();
    }

    fn rotate_y_static(forward: &mut Vector3<F>, up: &mut Vector3<F>, angle: F, snap: bool) {
        let axis_h = na::cross(forward, &AXIS_Z).normalize();

        if snap && false {
            let result_angle = (na::dot(&forward.clone(), &AXIS_Z)
                                / (na::distance(&na::origin(), &forward.to_point())
                                   * na::distance(&na::origin(), &AXIS_Z.to_point()) as F)).acos();

            println!("dot: {}", na::dot(&forward.clone(), &AXIS_Z).abs());
            println!("{} < {}", result_angle, angle);

            if result_angle < angle {
                // angle = result_angle;
                *forward = AXIS_Z;
                *up = na::cross(&axis_h, forward).normalize();
                return;
            } else if result_angle - std::f64::consts::PI as F > angle {
                // angle = result_angle - std::F::consts::PI;
                *forward = -AXIS_Z;
                *up = na::cross(&axis_h, forward).normalize();
                return;
            }
        }

        let quaternion = UnitQuaternion::new(axis_h * angle);
        *forward = quaternion.rotate(forward).normalize();
        *up = na::cross(&axis_h, forward).normalize();
    }

    fn rotate_x(&mut self, angle: F) {
        Camera3Impl::rotate_x_static(&mut self.forward, &mut self.up, angle);
    }

    fn rotate_y(&mut self, angle: F) {
        Camera3Impl::rotate_y_static(&mut self.forward, &mut self.up, angle, true);
    }

    fn get_right(&self) -> Vector3<F> {
        na::cross(&self.up, &self.forward).normalize()
    }

    fn get_left(&self) -> Vector3<F> {
        na::cross(&self.forward, &self.up).normalize()
    }
}

impl<F: BaseFloat> Camera<F, Point3<F>> for Camera3Impl<F> {
    #[allow(unused_variables)]
    fn get_ray_point(&self,
                     screen_x: i32,
                     screen_y: i32,
                     screen_width: i32,
                     screen_height: i32)
                     -> Point3<F> {
        self.location
    }

    fn get_ray_vector(&self,
                      screen_x: i32,
                      screen_y: i32,
                      screen_width: i32,
                      screen_height: i32)
                      -> Vector3<F> {
        let rel_x = (screen_x - screen_width / 2) as F + (1 - screen_width % 2) as F / 2.0;
        let rel_y = (screen_y - screen_height / 2) as F + (1 - screen_height % 2) as F / 2.0;
        let screen_width = screen_width as F;
        let screen_height = screen_height as F;
        let right = self.get_right();
        let fov_rad = std::f64::consts::PI as F * (self.fov as F) / 180.0;
        let distance_from_screen_center = (screen_width * screen_width + screen_height * screen_height).sqrt()
            / (2.0 * (fov_rad / 2.0).tan());
        let screen_center_point_3d = self.location + self.forward * distance_from_screen_center;
        let screen_point_3d = screen_center_point_3d + (self.up * rel_y) + (right * rel_x);

        (screen_point_3d - self.location).normalize()
    }
}

impl<F: BaseFloat> Entity<F, Point3<F>> for Camera3Impl<F> {
    fn as_updatable_mut(&mut self) -> Option<&mut Updatable<F, Point3<F>>> {
        Some(self)
    }

    fn as_updatable(&self) -> Option<&Updatable3<F>> {
        Some(self)
    }

    fn as_traceable_mut(&mut self) -> Option<&mut Traceable<F, Point3<F>>> {
        None
    }

    fn as_traceable(&self) -> Option<&Traceable3<F>> {
        None
    }
}

impl<F: BaseFloat> Updatable<F, Point3<F>> for Camera3Impl<F> {
    fn update(&mut self, delta_time: &Duration, context: &SimulationContext) {
        self.update_rotation(context);

        let pressed_keys: &HashSet<(u8, Option<VirtualKeyCode>)> = context.pressed_keys();
        let delta_millis = (delta_time.clone() * 1000u32).as_secs() as F / 1000.0;

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

impl<F: BaseFloat> Locatable<F, Point3<F>> for Camera3Impl<F> {
    fn location_mut(&mut self) -> &mut Point3<F> {
        &mut self.location
    }

    fn location(&self) -> &Point3<F> {
        &self.location
    }

    fn set_location(&mut self, location: Point3<F>) {
        self.location = location;
    }
}

impl<F: BaseFloat> Rotatable<F, Point3<F>> for Camera3Impl<F> {
    fn rotation_mut(&mut self) -> &mut Vector3<F> {
        &mut self.forward
    }

    fn rotation(&self) -> &Vector3<F> {
        &self.forward
    }

    fn set_rotation(&mut self, rotation: Vector3<F>) {
        self.forward = rotation;
    }
}
