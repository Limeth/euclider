use ::F;
use boolinator::Boolinator;
use glium::glutin::VirtualKeyCode;
use na::ApproxEq;
use na::BaseFloat;
use na::Cast;
use na::Matrix4;
use na::Transpose;
use na::Norm;
use na::Rotate;
use na::Eye;
use na::IterableMut;
use na;
use num::One;
use num::Zero;
use num::traits::NumCast;
use simulation::SimulationContext;
use std::collections::HashSet;
use std::time::Duration;
use universe::Universe;
use universe::d4::Point4;
use universe::d4::Universe4;
use universe::d4::Vector4;
use universe::d4::entity::*;
use universe::entity::Camera;
use universe::entity::Entity;
use universe::entity::Locatable;
use universe::entity::Rotatable;
use universe::entity::Traceable;
use util::AngleBetween;
use util::CustomFloat;
use util;

#[derive(Clone, Copy, PartialEq)]
pub struct FreeCamera4 {
    location: Point4,
    forward: Vector4,
    left: Vector4,
    up: Vector4,
    mouse_sensitivity: F,
    speed: F,
    fov: u8,
    max_depth: u32,
}

impl FreeCamera4 {
    pub fn new() -> FreeCamera4 {
        FreeCamera4 {
            location: na::origin(),
            forward: Vector4::new(<F as One>::one(), <F as Zero>::zero(),
                                  <F as Zero>::zero(), <F as Zero>::zero()),
            left: Vector4::y(),
            up: Vector4::z(),
            mouse_sensitivity: 0.01,
            speed: 10.0,
            fov: 90,
            max_depth: 10,
        }
    }

    pub fn new_with_location(location: Point4) -> Self {
        FreeCamera4 {
            location: location,
            .. Self::new()
        }
    }

    fn update_rotation(&mut self, delta_millis: F, context: &SimulationContext) {
        let pressed_keys: &HashSet<VirtualKeyCode> = context.pressed_keys();
        let mut angle: F = <F as Zero>::zero();

        pressed_keys.contains(&VirtualKeyCode::C).as_option()
            .map(|()| angle += <F as One>::one());
        pressed_keys.contains(&VirtualKeyCode::M).as_option()
            .map(|()| angle -= <F as One>::one());

        if angle != <F as Zero>::zero() {
            angle *= delta_millis * 2.0;
            let axis_array = [pressed_keys.contains(&VirtualKeyCode::I),  // X
                              pressed_keys.contains(&VirtualKeyCode::O),  // Y
                              pressed_keys.contains(&VirtualKeyCode::K),  // Z
                              pressed_keys.contains(&VirtualKeyCode::L)]; // W
            let axes_count = axis_array.iter().filter(|x| **x).count();

            if axes_count == 2 {
                let mut rotation_matrix = Matrix4::<F>::new_identity(4);

                for (index, value) in rotation_matrix.iter_mut().enumerate() {
                    let row = index / 4;
                    let column = index % 4;

                    if axis_array[row] && axis_array[column] {
                        *value = if row == column {
                            angle.cos()
                        } else if row < column {
                            -angle.sin()
                        } else {
                            angle.sin()
                        }
                    }
                }

                let to_ana = self.to_ana();
                let normalize_matrix = Matrix4::new(
                    self.forward.x, self.left.x, self.up.x, to_ana.x,
                    self.forward.y, self.left.y, self.up.y, to_ana.y,
                    self.forward.z, self.left.z, self.up.z, to_ana.z,
                    self.forward.w, self.left.w, self.up.w, to_ana.w
                );
                let transpose_normalize_matrix = normalize_matrix.transpose();

                {
                    let mut vectors_to_rotate = [&mut self.forward, &mut self.left, &mut self.up];

                    for vector_to_rotate in &mut vectors_to_rotate {
                        **vector_to_rotate = normalize_matrix * (rotation_matrix
                                                                 * (transpose_normalize_matrix * **vector_to_rotate));
                    }
                }

                let to_ana = self.to_ana();

                util::reorthonormalize_4(&mut self.forward, &mut self.left,
                                         &mut self.up, &to_ana);
            }
        }
    }

    /// Calculates the direction in the fourth axis.
    /// `Ana` stands for _north_ in this axis, `kata` is used for _south_.
    /// The method name `to_ana` stands for positive motion along this axis,
    /// similarly to how _up_ stands for the positive motion along the Z axis.
    fn to_ana(&self) -> Vector4 {
        util::find_orthonormal_4(&self.forward, &self.left, &self.up)
    }
}

impl Default for FreeCamera4 {
    fn default() -> Self {
        Self::new()
    }
}

impl Camera<Point4, Vector4, Universe4> for FreeCamera4 {
    #[allow(unused_variables)]
    fn get_ray_point(&self,
                     screen_x: i32,
                     screen_y: i32,
                     screen_width: i32,
                     screen_height: i32)
                     -> Point4 {
        self.location
    }

    fn get_ray_vector(&self,
                      screen_x: i32,
                      screen_y: i32,
                      screen_width: i32,
                      screen_height: i32)
                      -> Vector4 {
        let rel_x: F = <F as NumCast>::from(screen_x - screen_width / 2).unwrap() +
                       <F as NumCast>::from(1 - screen_width % 2).unwrap() / 2.0;
        let rel_y: F = <F as NumCast>::from(screen_y - screen_height / 2).unwrap() +
                       <F as NumCast>::from(1 - screen_height % 2).unwrap() / 2.0;
        let screen_width: F = <F as NumCast>::from(screen_width).unwrap();
        let screen_height: F = <F as NumCast>::from(screen_height).unwrap();
        let right = -self.left;
        let fov_rad: F = <F as BaseFloat>::pi() * <F as NumCast>::from(self.fov).unwrap() / 180.0;
        let distance_from_screen_center: F =
            (screen_width * screen_width + screen_height * screen_height).sqrt() /
            (<F as NumCast>::from(2.0).unwrap() * (fov_rad / 2.0).tan());
        let screen_center_point_4d = self.location + self.forward * distance_from_screen_center;
        let screen_point_4d = screen_center_point_4d + (self.up * rel_y) + (right * rel_x);

        (screen_point_4d - self.location).normalize()
    }

    fn max_depth(&self) -> u32 {
        self.max_depth
    }

    fn update(&mut self, delta_time: &Duration, context: &SimulationContext, universe: &Universe4) {
        let delta_millis = <F as NumCast>::from((*delta_time * 1000u32).as_secs()).unwrap() / 1000.0;

        self.update_rotation(delta_millis, context);

        let pressed_keys: &HashSet<VirtualKeyCode> = context.pressed_keys();
        let mut distance = self.speed * delta_millis;

        if distance == <F as Zero>::zero() {
            return;
        }

        let mut direction: Vector4 = na::zero();

        pressed_keys.contains(&VirtualKeyCode::W).as_option()
            .map(|()| direction += self.forward);
        pressed_keys.contains(&VirtualKeyCode::S).as_option()
            .map(|()| direction -= self.forward);
        pressed_keys.contains(&VirtualKeyCode::A).as_option()
            .map(|()| direction += self.left);
        pressed_keys.contains(&VirtualKeyCode::D).as_option()
            .map(|()| direction -= self.left);
        pressed_keys.contains(&VirtualKeyCode::LShift).as_option()
            .map(|()| direction += self.up);
        pressed_keys.contains(&VirtualKeyCode::LControl).as_option()
            .map(|()| direction -= self.up);
        pressed_keys.contains(&VirtualKeyCode::Q).as_option()
            .map(|()| direction += self.to_ana());
        pressed_keys.contains(&VirtualKeyCode::E).as_option()
            .map(|()| direction -= self.to_ana());

        if direction.norm_squared() != <F as Zero>::zero() {
            let length = direction.norm();
            distance *= length;

            direction.normalize_mut();

            if let Some((new_location, new_direction))
                    = universe.trace_path_unknown(delta_time,
                                                  &distance,
                                                  &self.location,
                                                  &direction,
                                                  context.debugging) {
                let rotation_scale = direction.angle_between(&new_direction);

                if !ApproxEq::approx_eq_ulps(&rotation_scale,
                                             &<F as Zero>::zero(),
                                             4_u32 * <F as ApproxEq<F>>::approx_ulps(None)) {
                    // TODO: Calculate the rotation that was applied to the vector
                    // and apply it to all the vectors that define the direction
                    // of the camera
                    unimplemented!();
                }

                self.location = new_location;
            }
        }
    }
}

impl Entity<Point4, Vector4> for FreeCamera4 {
    fn as_traceable_mut(&mut self) -> Option<&mut Traceable<Point4, Vector4>> {
        None
    }

    fn as_traceable(&self) -> Option<&Traceable4> {
        None
    }
}

impl Locatable<Point4, Vector4> for FreeCamera4 {
    fn location_mut(&mut self) -> &mut Point4 {
        &mut self.location
    }

    fn location(&self) -> &Point4 {
        &self.location
    }

    fn set_location(&mut self, location: Point4) {
        self.location = location;
    }
}

impl Rotatable<Point4, Vector4> for FreeCamera4 {
    fn rotation_mut(&mut self) -> &mut Vector4 {
        &mut self.forward
    }

    fn rotation(&self) -> &Vector4 {
        &self.forward
    }

    fn set_rotation(&mut self, rotation: Vector4) {
        self.forward = rotation;
    }
}
