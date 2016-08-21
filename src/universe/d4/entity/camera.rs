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
use universe::d4::entity::*;
use universe::d3::entity as d3_entity;
use num::traits::NumCast;
use num::Zero;
use num::One;
use simulation::SimulationContext;
use util::CustomFloat;
use util::AngleBetween;
use util::Derank;
use util::RankUp;

#[derive(Clone, Copy, PartialEq)]
pub struct Camera4Impl<F: CustomFloat> {
    location: Point4<F>,
    forward: Vector4<F>,
    up: Vector4<F>,
    mouse_sensitivity: F,
    speed: F,
    fov: u8,
    max_depth: u32,
}

unsafe impl<F: CustomFloat> Sync for Camera4Impl<F> {}

impl<F: CustomFloat> Camera4Impl<F> {
    pub fn new() -> Camera4Impl<F> {
        Camera4Impl {
            location: na::origin(),
            forward: Vector4::new(<F as One>::one(), <F as Zero>::zero(),
                                  <F as Zero>::zero(), <F as Zero>::zero()),
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
        let quaternion = UnitQuaternion::new(d3_entity::AXIS_Z() * angle);
        *forward = quaternion.rotate(forward).normalize();
        *up = quaternion.rotate(up).normalize();
    }

    fn rotate_y_static(forward: &mut Vector3<F>, up: &mut Vector3<F>, angle: F, snap: bool) {
        let axis_h = na::cross(forward, up).normalize();

        if snap {
            let result_angle = forward.angle_between(&d3_entity::AXIS_Z());

            if result_angle < angle {
                *forward = d3_entity::AXIS_Z();
                *up = na::cross(&axis_h, forward).normalize();
                return;
            } else if <F as BaseFloat>::pi() - result_angle < -angle {
                *forward = -d3_entity::AXIS_Z();
                *up = na::cross(&axis_h, forward).normalize();
                return;
            }
        }

        let quaternion = UnitQuaternion::new(axis_h * angle);
        *forward = quaternion.rotate(forward).normalize();
        *up = na::cross(&axis_h, forward).normalize();
    }

    fn rotate_x(&mut self, angle: F) {
        Camera4Impl::rotate_x_static(&mut self.forward.derank(), &mut self.up.derank(), angle);
    }

    fn rotate_y(&mut self, angle: F) {
        Camera4Impl::rotate_y_static(&mut self.forward.derank(), &mut self.up.derank(), angle, true);
    }

    fn get_left(&self) -> Vector3<F> {
        na::cross(&self.up.derank(), &self.forward.derank()).normalize()
    }

    fn get_right(&self) -> Vector3<F> {
        na::cross(&self.forward.derank(), &self.up.derank()).normalize()
    }
}

impl<F: CustomFloat> Camera<F, Point4<F>, Vector4<F>> for Camera4Impl<F> {
    #[allow(unused_variables)]
    fn get_ray_point(&self,
                     screen_x: i32,
                     screen_y: i32,
                     screen_width: i32,
                     screen_height: i32)
                     -> Point4<F> {
        self.location
    }

    fn get_ray_vector(&self,
                      screen_x: i32,
                      screen_y: i32,
                      screen_width: i32,
                      screen_height: i32)
                      -> Vector4<F> {
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
        let screen_point_3d = screen_center_point_3d + (self.up * rel_y) + (right.rankup() * rel_x);

        (screen_point_3d - self.location).normalize()
    }

    fn max_depth(&self) -> u32 {
        self.max_depth
    }

    fn update(&mut self, delta_time: &Duration, context: &SimulationContext, universe: &Universe<F, P=Point4<F>, V=Vector4<F>>) {
        self.update_rotation(context);

        let pressed_keys: &HashSet<(u8, Option<VirtualKeyCode>)> = context.pressed_keys();
        let delta_millis = <F as NumCast>::from((*delta_time * 1000u32).as_secs()).unwrap() /
                           Cast::from(1000.0);
        let distance = self.speed * delta_millis;

        if distance == <F as Zero>::zero() {
            return;
        }

        let mut direction: Vector3<F> = na::zero();

        for &(_, keycode) in pressed_keys {
            if let Some(keycode) = keycode {
                direction += match keycode {
                    VirtualKeyCode::W => self.forward.derank(),
                    VirtualKeyCode::S => -self.forward.derank(),
                    VirtualKeyCode::A => self.get_left(),
                    VirtualKeyCode::D => self.get_right(),
                    VirtualKeyCode::LControl => -d3_entity::AXIS_Z(),
                    VirtualKeyCode::LShift => d3_entity::AXIS_Z(),
                    _ => continue,
                };
            }
        }

        if direction.norm_squared() != <F as Zero>::zero() {
            if let Some((new_location, new_direction))
                    = universe.trace_path_unknown(delta_time,
                                                  &distance,
                                                  &self.location,
                                                  &direction.rankup()) {
                let rotation_scale = direction.angle_between(&new_direction.derank());

                if rotation_scale == <F as Zero>::zero() {
                    // TODO: Not tested, might need a lot of tuning.
                    let rotation_axis = na::cross(&direction, &new_direction.derank());
                    let difference = UnitQuaternion::new(rotation_axis * rotation_scale);
                    self.forward = difference.rotate(&self.forward.derank()).rankup();
                    self.up = difference.rotate(&self.up.derank()).rankup();
                }

                self.location = new_location;
            }
        }
    }
}

impl<F: CustomFloat> Entity<F, Point4<F>, Vector4<F>> for Camera4Impl<F> {
    fn as_traceable_mut(&mut self) -> Option<&mut Traceable<F, Point4<F>, Vector4<F>>> {
        None
    }

    fn as_traceable(&self) -> Option<&Traceable4<F>> {
        None
    }
}

impl<F: CustomFloat> Locatable<F, Point4<F>, Vector4<F>> for Camera4Impl<F> {
    fn location_mut(&mut self) -> &mut Point4<F> {
        &mut self.location
    }

    fn location(&self) -> &Point4<F> {
        &self.location
    }

    fn set_location(&mut self, location: Point4<F>) {
        self.location = location;
    }
}

impl<F: CustomFloat> Rotatable<F, Point4<F>, Vector4<F>> for Camera4Impl<F> {
    fn rotation_mut(&mut self) -> &mut Vector4<F> {
        &mut self.forward
    }

    fn rotation(&self) -> &Vector4<F> {
        &self.forward
    }

    fn set_rotation(&mut self, rotation: Vector4<F>) {
        self.forward = rotation;
    }
}
