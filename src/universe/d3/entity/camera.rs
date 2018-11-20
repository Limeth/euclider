use ::F;
use boolinator::Boolinator;
use glium::glutin::VirtualKeyCode;
use na;
use na::ApproxEq;
use na::BaseFloat;
use na::Cast;
use na::UnitQuaternion;
use na::Norm;
use na::Rotate;
use num::One;
use num::Zero;
use num::traits::NumCast;
use simulation::SimulationContext;
use std::collections::HashSet;
use std::time::Duration;
use universe::Universe;
use universe::d3::Point3;
use universe::d3::Universe3;
use universe::d3::Vector3;
use universe::d3::entity::*;
use universe::entity::Camera;
use universe::entity::Entity;
use universe::entity::Locatable;
use universe::entity::Rotatable;
use universe::entity::Traceable;
use util::AngleBetween;
use util::CustomFloat;

#[derive(Clone, Copy, PartialEq)]
pub struct Camera3Data {
    location: Point3,
    forward: Vector3,
    up: Vector3,
    mouse_sensitivity: F,
    speed: F,
    fov: u8,
    max_depth: u32,
}

impl Camera3Data {
    pub fn new() -> Self {
        Camera3Data {
            location: na::origin(),
            forward: Vector3::new(<F as One>::one(), <F as Zero>::zero(), <F as Zero>::zero()),
            up: Vector3::z(),
            mouse_sensitivity: 0.01,
            speed: 10.0,
            fov: 90,
            max_depth: 10,
        }
    }

    pub fn new_with_location(location: Point3) -> Self {
        Camera3Data {
            location: location,
            .. Self::new()
        }
    }

    fn get_left(&self) -> Vector3 {
        na::cross(&self.up, &self.forward).normalize()
    }

    fn get_right(&self) -> Vector3 {
        na::cross(&self.forward, &self.up).normalize()
    }
}

impl Default for Camera3Data {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Clone, Copy, PartialEq)]
pub struct PitchYawCamera3 {
    data: Camera3Data,
}

impl PitchYawCamera3 {
    pub fn new() -> PitchYawCamera3 {
        PitchYawCamera3 {
            data: Camera3Data::new(),
        }
    }

    pub fn new_with_location(location: Point3) -> Self {
        PitchYawCamera3 {
            data: Camera3Data::new_with_location(location)
        }
    }

    fn update_rotation(&mut self, context: &SimulationContext) {
        let delta_mouse_float: na::Vector2<F> =
            na::Vector2::<F>::new(<F as NumCast>::from(context.delta_mouse.x).unwrap(),
                                  <F as NumCast>::from(context.delta_mouse.y).unwrap());

        if na::distance_squared(&na::origin(), delta_mouse_float.as_point()) <=
           <F as Zero>::zero() {
            return;
        }

        let direction = delta_mouse_float * self.data.mouse_sensitivity;

        self.rotate_yaw(-direction.x);
        self.rotate_pitch(-direction.y);
    }

    fn rotate_yaw_static(forward: &mut Vector3, up: &mut Vector3, angle: F) {
        let quaternion = UnitQuaternion::new(Vector3::z() * angle);
        *forward = quaternion.rotate(forward).normalize();
        *up = quaternion.rotate(up).normalize();
    }

    fn rotate_pitch_static(forward: &mut Vector3, up: &mut Vector3, angle: F, snap: bool) {
        let axis_h = na::cross(forward, up).normalize();

        if snap {
            let result_angle = forward.angle_between(&Vector3::z());

            if result_angle < angle {
                *forward = Vector3::z();
                *up = na::cross(&axis_h, forward).normalize();
                return;
            } else if <F as BaseFloat>::pi() - result_angle < -angle {
                *forward = -Vector3::z();
                *up = na::cross(&axis_h, forward).normalize();
                return;
            }
        }

        let quaternion = UnitQuaternion::new(axis_h * angle);
        *forward = quaternion.rotate(forward).normalize();
        *up = na::cross(&axis_h, forward).normalize();
    }

    fn rotate_yaw(&mut self, angle: F) {
        Self::rotate_yaw_static(&mut self.data.forward, &mut self.data.up, angle);
    }

    fn rotate_pitch(&mut self, angle: F) {
        Self::rotate_pitch_static(&mut self.data.forward, &mut self.data.up, angle, true);
    }
}

impl Default for PitchYawCamera3 {
    fn default() -> Self {
        Self::new()
    }
}

impl Camera<Point3, Vector3, Universe3> for PitchYawCamera3 {
    #[allow(unused_variables)]
    fn get_ray_point(&self,
                     screen_x: i32,
                     screen_y: i32,
                     screen_width: i32,
                     screen_height: i32)
                     -> Point3 {
        self.data.location
    }

    fn get_ray_vector(&self,
                      screen_x: i32,
                      screen_y: i32,
                      screen_width: i32,
                      screen_height: i32)
                      -> Vector3 {
        let rel_x: F = <F as NumCast>::from(screen_x - screen_width / 2).unwrap() +
                       <F as NumCast>::from(1 - screen_width % 2).unwrap() / 2.0;
        let rel_y: F = <F as NumCast>::from(screen_y - screen_height / 2).unwrap() +
                       <F as NumCast>::from(1 - screen_height % 2).unwrap() / 2.0;
        let screen_width: F = <F as NumCast>::from(screen_width).unwrap();
        let screen_height: F = <F as NumCast>::from(screen_height).unwrap();
        let right = self.data.get_right();
        let fov_rad: F = <F as BaseFloat>::pi() * <F as NumCast>::from(self.data.fov).unwrap() / 180.0;
        let distance_from_screen_center: F =
            (screen_width * screen_width + screen_height * screen_height).sqrt() /
            (2.0 * (fov_rad / 2.0).tan());
        let screen_center_point_3d = self.data.location + self.data.forward * distance_from_screen_center;
        let screen_point_3d = screen_center_point_3d + (self.data.up * rel_y) + (right * rel_x);

        (screen_point_3d - self.data.location).normalize()
    }

    fn max_depth(&self) -> u32 {
        self.data.max_depth
    }

    fn update(&mut self, delta_time: &Duration, context: &SimulationContext, universe: &Universe3) {
        self.update_rotation(context);

        let pressed_keys: &HashSet<VirtualKeyCode> = context.pressed_keys();
        let delta_millis = <F as NumCast>::from((*delta_time * 1000u32).as_secs()).unwrap() / 1000.0;
        let mut distance = self.data.speed * delta_millis;

        if distance == <F as Zero>::zero() {
            return;
        }

        let mut direction: Vector3 = na::zero();

        pressed_keys.contains(&VirtualKeyCode::W).as_option()
            .map(|()| direction += self.data.forward);
        pressed_keys.contains(&VirtualKeyCode::S).as_option()
            .map(|()| direction -= self.data.forward);
        pressed_keys.contains(&VirtualKeyCode::A).as_option()
            .map(|()| direction += self.data.get_left());
        pressed_keys.contains(&VirtualKeyCode::D).as_option()
            .map(|()| direction -= self.data.get_left());
        pressed_keys.contains(&VirtualKeyCode::LShift).as_option()
            .map(|()| direction += Vector3::z());
        pressed_keys.contains(&VirtualKeyCode::LControl).as_option()
            .map(|()| direction -= Vector3::z());

        if direction.norm_squared() != <F as Zero>::zero() {
            let length = direction.norm();
            distance *= length;

            direction.normalize_mut();

            if let Some((new_location, new_direction))
                    = universe.trace_path_unknown(delta_time,
                                                  &distance,
                                                  &self.data.location,
                                                  &direction,
                                                  context.debugging) {
                let rotation_scale = direction.angle_between(&new_direction);

                if !ApproxEq::approx_eq_ulps(&rotation_scale,
                                             &<F as Zero>::zero(),
                                             <F as ApproxEq<F>>::approx_ulps(None)) {
                    // TODO: Not tested, might need a lot of tuning.
                    let rotation_axis = na::cross(&direction, &new_direction);
                    let difference = UnitQuaternion::new(rotation_axis * rotation_scale);
                    self.data.forward = difference.rotate(&self.data.forward);
                    self.data.up = difference.rotate(&self.data.up);
                }

                self.data.location = new_location;
            }
        }
    }
}

impl Entity<Point3, Vector3> for PitchYawCamera3 {
    fn as_traceable_mut(&mut self) -> Option<&mut Traceable<Point3, Vector3>> {
        None
    }

    fn as_traceable(&self) -> Option<&Traceable3> {
        None
    }
}

impl Locatable<Point3, Vector3> for PitchYawCamera3 {
    fn location_mut(&mut self) -> &mut Point3 {
        &mut self.data.location
    }

    fn location(&self) -> &Point3 {
        &self.data.location
    }

    fn set_location(&mut self, location: Point3) {
        self.data.location = location;
    }
}

impl Rotatable<Point3, Vector3> for PitchYawCamera3 {
    fn rotation_mut(&mut self) -> &mut Vector3 {
        &mut self.data.forward
    }

    fn rotation(&self) -> &Vector3 {
        &self.data.forward
    }

    fn set_rotation(&mut self, rotation: Vector3) {
        self.data.forward = rotation;
    }
}

#[derive(Clone, Copy, PartialEq)]
pub struct FreeCamera3 {
    data: Camera3Data,
}

impl FreeCamera3 {
    pub fn new() -> FreeCamera3 {
        FreeCamera3 {
            data: Camera3Data::new(),
        }
    }

    pub fn new_with_location(location: Point3) -> Self {
        FreeCamera3 {
            data: Camera3Data::new_with_location(location)
        }
    }

    fn update_rotation(&mut self, delta_millis: F, context: &SimulationContext) {
        let delta_mouse_float: na::Vector2<F> =
            na::Vector2::<F>::new(<F as NumCast>::from(context.delta_mouse.x).unwrap(),
                         <F as NumCast>::from(context.delta_mouse.y).unwrap());
        let pressed_keys: &HashSet<VirtualKeyCode> = context.pressed_keys();
        let direction = delta_mouse_float * self.data.mouse_sensitivity;
        let mut roll = <F as Zero>::zero();
        pressed_keys.contains(&VirtualKeyCode::Q).as_option()
            .map(|()| roll -= <F as One>::one());
        pressed_keys.contains(&VirtualKeyCode::E).as_option()
            .map(|()| roll += <F as One>::one());
        roll *= delta_millis * 2.0;

        if direction.x != <F as Zero>::zero() {
            self.rotate_yaw(-direction.x);
        }

        if direction.y != <F as Zero>::zero() {
            self.rotate_pitch(-direction.y);
        }

        if roll != <F as Zero>::zero() {
            self.rotate_roll(roll);
        }
    }

    fn rotate_yaw_static(forward: &mut Vector3, up: &mut Vector3, angle: F) {
        let quaternion = UnitQuaternion::new(*up * angle);
        *forward = quaternion.rotate(forward).normalize();
    }

    fn rotate_roll_static(forward: &mut Vector3, up: &mut Vector3, angle: F) {
        let quaternion = UnitQuaternion::new(*forward * angle);
        *up = quaternion.rotate(up).normalize();
    }

    fn rotate_yaw(&mut self, angle: F) {
        Self::rotate_yaw_static(&mut self.data.forward, &mut self.data.up, angle);
    }

    fn rotate_pitch(&mut self, angle: F) {
        PitchYawCamera3::rotate_pitch_static(&mut self.data.forward, &mut self.data.up, angle, false);
    }

    fn rotate_roll(&mut self, angle: F) {
        Self::rotate_roll_static(&mut self.data.forward, &mut self.data.up, angle);
    }
}

impl Default for FreeCamera3 {
    fn default() -> Self {
        Self::new()
    }
}

impl Camera<Point3, Vector3, Universe3> for FreeCamera3 {
    #[allow(unused_variables)]
    fn get_ray_point(&self,
                     screen_x: i32,
                     screen_y: i32,
                     screen_width: i32,
                     screen_height: i32)
                     -> Point3 {
        self.data.location
    }

    fn get_ray_vector(&self,
                      screen_x: i32,
                      screen_y: i32,
                      screen_width: i32,
                      screen_height: i32)
                      -> Vector3 {
        let rel_x: F = <F as NumCast>::from(screen_x - screen_width / 2).unwrap() +
                       <F as NumCast>::from(1 - screen_width % 2).unwrap() / 2.0;
        let rel_y: F = <F as NumCast>::from(screen_y - screen_height / 2).unwrap() +
                       <F as NumCast>::from(1 - screen_height % 2).unwrap() / 2.0;
        let screen_width: F = <F as NumCast>::from(screen_width).unwrap();
        let screen_height: F = <F as NumCast>::from(screen_height).unwrap();
        let right = self.data.get_right();
        let fov_rad: F = <F as BaseFloat>::pi() * <F as NumCast>::from(self.data.fov).unwrap() / 180.0;
        let distance_from_screen_center: F =
            (screen_width * screen_width + screen_height * screen_height).sqrt() /
            (<F as NumCast>::from(2.0).unwrap() * (fov_rad / 2.0).tan());
        let screen_center_point_3d = self.data.location + self.data.forward * distance_from_screen_center;
        let screen_point_3d = screen_center_point_3d + (self.data.up * rel_y) + (right * rel_x);

        (screen_point_3d - self.data.location).normalize()
    }

    fn max_depth(&self) -> u32 {
        self.data.max_depth
    }

    fn update(&mut self, delta_time: &Duration, context: &SimulationContext, universe: &Universe3) {
        let delta_millis = <F as NumCast>::from((*delta_time * 1000u32).as_secs()).unwrap() / 1000.0;

        self.update_rotation(delta_millis, context);

        let pressed_keys: &HashSet<VirtualKeyCode> = context.pressed_keys();
        let mut distance = self.data.speed * delta_millis;

        if distance == <F as Zero>::zero() {
            return;
        }

        let mut direction: Vector3 = na::zero();

        pressed_keys.contains(&VirtualKeyCode::W).as_option()
            .map(|()| direction += self.data.forward);
        pressed_keys.contains(&VirtualKeyCode::S).as_option()
            .map(|()| direction -= self.data.forward);
        pressed_keys.contains(&VirtualKeyCode::A).as_option()
            .map(|()| direction += self.data.get_left());
        pressed_keys.contains(&VirtualKeyCode::D).as_option()
            .map(|()| direction -= self.data.get_left());
        pressed_keys.contains(&VirtualKeyCode::LShift).as_option()
            .map(|()| direction += self.data.up);
        pressed_keys.contains(&VirtualKeyCode::LControl).as_option()
            .map(|()| direction -= self.data.up);

        if direction.norm_squared() != <F as Zero>::zero() {
            let length = direction.norm();
            distance *= length;

            direction.normalize_mut();

            if let Some((new_location, new_direction))
                    = universe.trace_path_unknown(delta_time,
                                                  &distance,
                                                  &self.data.location,
                                                  &direction,
                                                  context.debugging) {
                let rotation_scale = direction.angle_between(&new_direction);

                if !ApproxEq::<F>::approx_eq_ulps(&rotation_scale,
                                             &<F as Zero>::zero(),
                                             <F as ApproxEq<F>>::approx_ulps(None)) {
                    // TODO: Not tested, might need a lot of tuning.
                    let rotation_axis = na::cross(&direction, &new_direction);
                    let difference = UnitQuaternion::new(rotation_axis * rotation_scale);
                    self.data.forward = difference.rotate(&self.data.forward);
                    self.data.up = difference.rotate(&self.data.up);
                }

                self.data.location = new_location;
            }
        }
    }
}

impl Entity<Point3, Vector3> for FreeCamera3 {
    fn as_traceable_mut(&mut self) -> Option<&mut Traceable<Point3, Vector3>> {
        None
    }

    fn as_traceable(&self) -> Option<&Traceable3> {
        None
    }
}

impl Locatable<Point3, Vector3> for FreeCamera3 {
    fn location_mut(&mut self) -> &mut Point3 {
        &mut self.data.location
    }

    fn location(&self) -> &Point3 {
        &self.data.location
    }

    fn set_location(&mut self, location: Point3) {
        self.data.location = location;
    }
}

impl Rotatable<Point3, Vector3> for FreeCamera3 {
    fn rotation_mut(&mut self) -> &mut Vector3 {
        &mut self.data.forward
    }

    fn rotation(&self) -> &Vector3 {
        &self.data.forward
    }

    fn set_rotation(&mut self, rotation: Vector3) {
        self.data.forward = rotation;
    }
}
