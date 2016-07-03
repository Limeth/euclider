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
use num::traits::NumCast;
use num::Zero;
use num::One;
use SimulationContext;
use util::CustomFloat;

#[derive(Clone, Copy, PartialEq)]
pub struct Camera3Impl<F: CustomFloat> {
    location: Point3<F>,
    forward: Vector3<F>,
    up: Vector3<F>,
    mouse_sensitivity: F,
    speed: F,
    fov: u8,
    max_depth: u32,
}

unsafe impl<F: CustomFloat> Sync for Camera3Impl<F> {}

impl<F: CustomFloat> Camera3Impl<F> {
    pub fn new() -> Camera3Impl<F> {
        Camera3Impl {
            location: na::origin(),
            forward: Vector3::new(<F as One>::one(), <F as Zero>::zero(), <F as Zero>::zero()),
            up: AXIS_Z(),
            mouse_sensitivity: Cast::from(0.01),
            speed: Cast::from(10.0),
            fov: 90,
            max_depth: 10,
        }
    }

    fn update_rotation(&mut self, context: &SimulationContext) {
        let delta_mouse_float: Vector2<F> =
            Vector2::new(<F as NumCast>::from(context.delta_mouse.x).unwrap(),
                         <F as NumCast>::from(context.delta_mouse.y).unwrap());

        if na::distance_squared(&na::origin(), delta_mouse_float.as_point()) <=
           <F as Zero>::zero() {
            return;
        }

        let direction = delta_mouse_float * self.mouse_sensitivity;

        self.rotate_x(-direction.x);
        self.rotate_y(-direction.y);
    }

    fn rotate_x_static(forward: &mut Vector3<F>, up: &mut Vector3<F>, angle: F) {
        let axis_h = na::cross(forward, &AXIS_Z()).normalize();
        let quaternion = UnitQuaternion::new(AXIS_Z() * angle);
        *forward = quaternion.rotate(forward).normalize();
        *up = na::cross(&axis_h, forward).normalize();
    }

    fn rotate_y_static(forward: &mut Vector3<F>, up: &mut Vector3<F>, angle: F/*, snap: bool*/) {
        let axis_h = na::cross(forward, &AXIS_Z()).normalize();

        // TODO snap to the Z axis
        // if snap {
        //     let result_angle = (na::dot(&forward.clone(), &AXIS_Z()) /
        //                         (na::distance(&na::origin(), &forward.to_point()) *
        //                          na::distance(&na::origin(), &AXIS_Z().to_point())))
        //         .acos();

        //     println!("dot: {}", na::dot(&forward.clone(), &AXIS_Z()).abs());
        //     println!("{} < {}", result_angle, angle);

        //     if result_angle < angle {
        //         // angle = result_angle;
        //         *forward = AXIS_Z();
        //         *up = na::cross(&axis_h, forward).normalize();
        //         return;
        //     } else if result_angle - <F as BaseFloat>::pi() > angle {
        //         // angle = result_angle - std::F::consts::PI;
        //         *forward = -AXIS_Z();
        //         *up = na::cross(&axis_h, forward).normalize();
        //         return;
        //     }
        // }

        let quaternion = UnitQuaternion::new(axis_h * angle);
        *forward = quaternion.rotate(forward).normalize();
        *up = na::cross(&axis_h, forward).normalize();
    }

    fn rotate_x(&mut self, angle: F) {
        Camera3Impl::rotate_x_static(&mut self.forward, &mut self.up, angle);
    }

    fn rotate_y(&mut self, angle: F) {
        Camera3Impl::rotate_y_static(&mut self.forward, &mut self.up, angle/*, true*/);
    }

    fn get_left(&self) -> Vector3<F> {
        na::cross(&self.up, &self.forward).normalize()
    }

    fn get_right(&self) -> Vector3<F> {
        na::cross(&self.forward, &self.up).normalize()
    }
}

impl<F: CustomFloat> Camera<F, Point3<F>, Vector3<F>> for Camera3Impl<F> {
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
        let rel_x: F = <F as NumCast>::from(screen_x - screen_width / 2).unwrap() +
                       <F as NumCast>::from(1 - screen_width % 2).unwrap() / Cast::from(2.0);
        let rel_y: F = <F as NumCast>::from(screen_y - screen_height / 2).unwrap() +
                       <F as NumCast>::from(1 - screen_height % 2).unwrap() / Cast::from(2.0);
        let screen_width: F = <F as NumCast>::from(screen_width).unwrap();
        let screen_height: F = <F as NumCast>::from(screen_height).unwrap();
        let right = self.get_right();
        let fov_rad: F = <F as BaseFloat>::pi() * <F as NumCast>::from(self.fov).unwrap() /
                         Cast::from(180.0);
        let distance_from_screen_center: F =
            (screen_width * screen_width + screen_height * screen_height).sqrt() /
            (<F as NumCast>::from(2.0).unwrap() * (fov_rad / Cast::from(2.0)).tan());
        let screen_center_point_3d = self.location + self.forward * distance_from_screen_center;
        let screen_point_3d = screen_center_point_3d + (self.up * rel_y) + (right * rel_x);

        (screen_point_3d - self.location).normalize()
    }

    fn max_depth(&self) -> u32 {
        self.max_depth
    }
}

impl<F: CustomFloat> Entity<F, Point3<F>, Vector3<F>> for Camera3Impl<F> {
    fn as_updatable_mut(&mut self) -> Option<&mut Updatable<F, Point3<F>, Vector3<F>>> {
        Some(self)
    }

    fn as_updatable(&self) -> Option<&Updatable3<F>> {
        Some(self)
    }

    fn as_traceable_mut(&mut self) -> Option<&mut Traceable<F, Point3<F>, Vector3<F>>> {
        None
    }

    fn as_traceable(&self) -> Option<&Traceable3<F>> {
        None
    }
}

impl<F: CustomFloat> Updatable<F, Point3<F>, Vector3<F>> for Camera3Impl<F> {
    fn update(&mut self, delta_time: &Duration, context: &SimulationContext) {
        self.update_rotation(context);

        let pressed_keys: &HashSet<(u8, Option<VirtualKeyCode>)> = context.pressed_keys();
        let delta_millis = <F as NumCast>::from((*delta_time * 1000u32).as_secs())
            .unwrap() / Cast::from(1000.0);

        for pressed_key in pressed_keys {
            if pressed_key.1.is_none() {
                continue;
            }

            match pressed_key.1.unwrap() {
                VirtualKeyCode::W => {
                    self.location += self.forward * self.speed * delta_millis;
                }
                VirtualKeyCode::S => {
                    self.location += -self.forward * self.speed * delta_millis;
                }
                VirtualKeyCode::A => {
                    self.location += self.get_left() * self.speed * delta_millis;
                }
                VirtualKeyCode::D => {
                    self.location += self.get_right() * self.speed * delta_millis;
                }
                VirtualKeyCode::LControl => {
                    self.location += -AXIS_Z() * self.speed * delta_millis;
                }
                VirtualKeyCode::LShift => {
                    self.location += AXIS_Z() * self.speed * delta_millis;
                }
                _ => (),
            }
        }
    }
}

impl<F: CustomFloat> Locatable<F, Point3<F>, Vector3<F>> for Camera3Impl<F> {
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

impl<F: CustomFloat> Rotatable<F, Point3<F>, Vector3<F>> for Camera3Impl<F> {
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
