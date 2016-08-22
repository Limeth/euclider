use std::collections::HashSet;
use std::time::Duration;
use na;
use na::*;
use glium::glutin::VirtualKeyCode;
use universe::Universe;
use universe::entity::Entity;
use universe::entity::Locatable;
use universe::entity::Rotatable;
use universe::entity::Traceable;
use universe::entity::Camera;
use universe::d3::entity::*;
use num::traits::NumCast;
use num::Zero;
use num::One;
use simulation::SimulationContext;
use util::CustomFloat;
use util::AngleBetween;

#[derive(Clone, Copy, PartialEq)]
pub struct Camera3Data<F: CustomFloat> {
    location: Point3<F>,
    forward: Vector3<F>,
    up: Vector3<F>,
    mouse_sensitivity: F,
    speed: F,
    fov: u8,
    max_depth: u32,
}

impl<F: CustomFloat> Camera3Data<F> {
    pub fn new() -> Self {
        Camera3Data {
            location: na::origin(),
            forward: Vector3::new(<F as One>::one(), <F as Zero>::zero(), <F as Zero>::zero()),
            up: AXIS_Z(),
            mouse_sensitivity: Cast::from(0.01),
            speed: Cast::from(10.0),
            fov: 90,
            max_depth: 10,
        }
    }

    fn get_left(&self) -> Vector3<F> {
        na::cross(&self.up, &self.forward).normalize()
    }

    fn get_right(&self) -> Vector3<F> {
        na::cross(&self.forward, &self.up).normalize()
    }
}

#[derive(Clone, Copy, PartialEq)]
pub struct PitchYawCamera3<F: CustomFloat> {
    data: Camera3Data<F>,
}

unsafe impl<F: CustomFloat> Sync for PitchYawCamera3<F> {}

impl<F: CustomFloat> PitchYawCamera3<F> {
    pub fn new() -> PitchYawCamera3<F> {
        PitchYawCamera3 {
            data: Camera3Data::new(),
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

        let direction = delta_mouse_float * self.data.mouse_sensitivity;

        self.rotate_yaw(-direction.x);
        self.rotate_pitch(-direction.y);
    }

    fn rotate_yaw_static(forward: &mut Vector3<F>, up: &mut Vector3<F>, angle: F) {
        let quaternion = UnitQuaternion::new(AXIS_Z() * angle);
        *forward = quaternion.rotate(forward).normalize();
        *up = quaternion.rotate(up).normalize();
    }

    fn rotate_pitch_static(forward: &mut Vector3<F>, up: &mut Vector3<F>, angle: F, snap: bool) {
        let axis_h = na::cross(forward, up).normalize();

        if snap {
            let result_angle = forward.angle_between(&AXIS_Z());

            if result_angle < angle {
                *forward = AXIS_Z();
                *up = na::cross(&axis_h, forward).normalize();
                return;
            } else if <F as BaseFloat>::pi() - result_angle < -angle {
                *forward = -AXIS_Z();
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

impl<F: CustomFloat> Camera<F, Point3<F>, Vector3<F>> for PitchYawCamera3<F> {
    #[allow(unused_variables)]
    fn get_ray_point(&self,
                     screen_x: i32,
                     screen_y: i32,
                     screen_width: i32,
                     screen_height: i32)
                     -> Point3<F> {
        self.data.location
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
        let right = self.data.get_right();
        let fov_rad: F = <F as BaseFloat>::pi() * <F as NumCast>::from(self.data.fov).unwrap() /
                         Cast::from(180.0);
        let distance_from_screen_center: F =
            (screen_width * screen_width + screen_height * screen_height).sqrt() /
            (<F as NumCast>::from(2.0).unwrap() * (fov_rad / Cast::from(2.0)).tan());
        let screen_center_point_3d = self.data.location + self.data.forward * distance_from_screen_center;
        let screen_point_3d = screen_center_point_3d + (self.data.up * rel_y) + (right * rel_x);

        (screen_point_3d - self.data.location).normalize()
    }

    fn max_depth(&self) -> u32 {
        self.data.max_depth
    }

    fn update(&mut self, delta_time: &Duration, context: &SimulationContext, universe: &Universe<F, P=Point3<F>, V=Vector3<F>>) {
        self.update_rotation(context);

        let pressed_keys: &HashSet<(u8, Option<VirtualKeyCode>)> = context.pressed_keys();
        let delta_millis = <F as NumCast>::from((*delta_time * 1000u32).as_secs()).unwrap() /
                           Cast::from(1000.0);
        let distance = self.data.speed * delta_millis;

        if distance == <F as Zero>::zero() {
            return;
        }

        let mut direction: Vector3<F> = na::zero();

        for &(_, keycode) in pressed_keys {
            if let Some(keycode) = keycode {
                direction += match keycode {
                    VirtualKeyCode::W => self.data.forward,
                    VirtualKeyCode::S => -self.data.forward,
                    VirtualKeyCode::A => self.data.get_left(),
                    VirtualKeyCode::D => self.data.get_right(),
                    VirtualKeyCode::LControl => -AXIS_Z(),
                    VirtualKeyCode::LShift => AXIS_Z(),
                    _ => continue,
                };
            }
        }

        if direction.norm_squared() != <F as Zero>::zero() {
            if let Some((new_location, new_direction))
                    = universe.trace_path_unknown(delta_time,
                                                  &distance,
                                                  &self.data.location,
                                                  &direction) {
                let rotation_scale = direction.angle_between(&new_direction);

                if rotation_scale == <F as Zero>::zero() {
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

impl<F: CustomFloat> Entity<F, Point3<F>, Vector3<F>> for PitchYawCamera3<F> {
    fn as_traceable_mut(&mut self) -> Option<&mut Traceable<F, Point3<F>, Vector3<F>>> {
        None
    }

    fn as_traceable(&self) -> Option<&Traceable3<F>> {
        None
    }
}

impl<F: CustomFloat> Locatable<F, Point3<F>, Vector3<F>> for PitchYawCamera3<F> {
    fn location_mut(&mut self) -> &mut Point3<F> {
        &mut self.data.location
    }

    fn location(&self) -> &Point3<F> {
        &self.data.location
    }

    fn set_location(&mut self, location: Point3<F>) {
        self.data.location = location;
    }
}

impl<F: CustomFloat> Rotatable<F, Point3<F>, Vector3<F>> for PitchYawCamera3<F> {
    fn rotation_mut(&mut self) -> &mut Vector3<F> {
        &mut self.data.forward
    }

    fn rotation(&self) -> &Vector3<F> {
        &self.data.forward
    }

    fn set_rotation(&mut self, rotation: Vector3<F>) {
        self.data.forward = rotation;
    }
}

#[derive(Clone, Copy, PartialEq)]
pub struct FreeCamera3<F: CustomFloat> {
    data: Camera3Data<F>,
}

unsafe impl<F: CustomFloat> Sync for FreeCamera3<F> {}

impl<F: CustomFloat> FreeCamera3<F> {
    pub fn new() -> FreeCamera3<F> {
        FreeCamera3 {
            data: Camera3Data::new(),
        }
    }

    fn update_rotation(&mut self, delta_millis: F, context: &SimulationContext) {
        let delta_mouse_float: Vector2<F> =
            Vector2::new(<F as NumCast>::from(context.delta_mouse.x).unwrap(),
                         <F as NumCast>::from(context.delta_mouse.y).unwrap());
        let pressed_keys: &HashSet<(u8, Option<VirtualKeyCode>)> = context.pressed_keys();
        let direction = delta_mouse_float * self.data.mouse_sensitivity;
        let mut roll = <F as Zero>::zero();

        for &(_, keycode) in pressed_keys {
            if let Some(keycode) = keycode {
                roll += match keycode {
                    VirtualKeyCode::Q => -<F as One>::one(),
                    VirtualKeyCode::E => <F as One>::one(),
                    _ => continue,
                };
            }
        }

        roll *= delta_millis * Cast::from(2.0);

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

    fn rotate_yaw_static(forward: &mut Vector3<F>, up: &mut Vector3<F>, angle: F) {
        let quaternion = UnitQuaternion::new(*up * angle);
        *forward = quaternion.rotate(forward).normalize();
    }

    fn rotate_roll_static(forward: &mut Vector3<F>, up: &mut Vector3<F>, angle: F) {
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

impl<F: CustomFloat> Camera<F, Point3<F>, Vector3<F>> for FreeCamera3<F> {
    #[allow(unused_variables)]
    fn get_ray_point(&self,
                     screen_x: i32,
                     screen_y: i32,
                     screen_width: i32,
                     screen_height: i32)
                     -> Point3<F> {
        self.data.location
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
        let right = self.data.get_right();
        let fov_rad: F = <F as BaseFloat>::pi() * <F as NumCast>::from(self.data.fov).unwrap() /
                         Cast::from(180.0);
        let distance_from_screen_center: F =
            (screen_width * screen_width + screen_height * screen_height).sqrt() /
            (<F as NumCast>::from(2.0).unwrap() * (fov_rad / Cast::from(2.0)).tan());
        let screen_center_point_3d = self.data.location + self.data.forward * distance_from_screen_center;
        let screen_point_3d = screen_center_point_3d + (self.data.up * rel_y) + (right * rel_x);

        (screen_point_3d - self.data.location).normalize()
    }

    fn max_depth(&self) -> u32 {
        self.data.max_depth
    }

    fn update(&mut self, delta_time: &Duration, context: &SimulationContext, universe: &Universe<F, P=Point3<F>, V=Vector3<F>>) {
        let delta_millis = <F as NumCast>::from((*delta_time * 1000u32).as_secs()).unwrap() /
                           Cast::from(1000.0);

        self.update_rotation(delta_millis, context);

        let pressed_keys: &HashSet<(u8, Option<VirtualKeyCode>)> = context.pressed_keys();
        let distance = self.data.speed * delta_millis;

        if distance == <F as Zero>::zero() {
            return;
        }

        let mut direction: Vector3<F> = na::zero();

        for &(_, keycode) in pressed_keys {
            if let Some(keycode) = keycode {
                direction += match keycode {
                    VirtualKeyCode::W => self.data.forward,
                    VirtualKeyCode::S => -self.data.forward,
                    VirtualKeyCode::A => self.data.get_left(),
                    VirtualKeyCode::D => self.data.get_right(),
                    VirtualKeyCode::LControl => -self.data.up,
                    VirtualKeyCode::LShift => self.data.up,
                    _ => continue,
                };
            }
        }

        if direction.norm_squared() != <F as Zero>::zero() {
            if let Some((new_location, new_direction))
                    = universe.trace_path_unknown(delta_time,
                                                  &distance,
                                                  &self.data.location,
                                                  &direction) {
                let rotation_scale = direction.angle_between(&new_direction);

                if rotation_scale == <F as Zero>::zero() {
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

impl<F: CustomFloat> Entity<F, Point3<F>, Vector3<F>> for FreeCamera3<F> {
    fn as_traceable_mut(&mut self) -> Option<&mut Traceable<F, Point3<F>, Vector3<F>>> {
        None
    }

    fn as_traceable(&self) -> Option<&Traceable3<F>> {
        None
    }
}

impl<F: CustomFloat> Locatable<F, Point3<F>, Vector3<F>> for FreeCamera3<F> {
    fn location_mut(&mut self) -> &mut Point3<F> {
        &mut self.data.location
    }

    fn location(&self) -> &Point3<F> {
        &self.data.location
    }

    fn set_location(&mut self, location: Point3<F>) {
        self.data.location = location;
    }
}

impl<F: CustomFloat> Rotatable<F, Point3<F>, Vector3<F>> for FreeCamera3<F> {
    fn rotation_mut(&mut self) -> &mut Vector3<F> {
        &mut self.data.forward
    }

    fn rotation(&self) -> &Vector3<F> {
        &self.data.forward
    }

    fn set_rotation(&mut self, rotation: Vector3<F>) {
        self.data.forward = rotation;
    }
}
