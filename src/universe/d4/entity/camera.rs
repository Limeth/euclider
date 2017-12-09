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
use universe::d4::Universe4;
use universe::d4::entity::*;
use num::traits::NumCast;
use num::Zero;
use num::One;
use simulation::SimulationContext;
use util;
use util::CustomFloat;
use util::AngleBetween;
use boolinator::Boolinator;

#[derive(Clone, Copy, PartialEq)]
pub struct FreeCamera4<F: CustomFloat> {
    location: Point4<F>,
    forward: Vector4<F>,
    left: Vector4<F>,
    up: Vector4<F>,
    mouse_sensitivity: F,
    speed: F,
    fov: u8,
    max_depth: u32,
}

unsafe impl<F: CustomFloat> Sync for FreeCamera4<F> {}

impl<F: CustomFloat> FreeCamera4<F> {
    pub fn new() -> FreeCamera4<F> {
        FreeCamera4 {
            location: na::origin(),
            forward: Vector4::new(<F as One>::one(), <F as Zero>::zero(),
                                  <F as Zero>::zero(), <F as Zero>::zero()),
            left: Vector4::y(),
            up: Vector4::z(),
            mouse_sensitivity: Cast::from(0.01),
            speed: Cast::from(10.0),
            fov: 90,
            max_depth: 10,
        }
    }

    pub fn new_with_location(location: Point4<F>) -> Self {
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
            angle *= delta_millis * Cast::from(2.0);
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
    fn to_ana(&self) -> Vector4<F> {
        util::find_orthonormal_4(&self.forward, &self.left, &self.up)
    }
}

impl<F: CustomFloat> Default for FreeCamera4<F> {
    fn default() -> Self {
        Self::new()
    }
}

impl<F: CustomFloat> Camera<F, Point4<F>, Vector4<F>, Universe4<F>> for FreeCamera4<F> {
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
        let right = -self.left;
        let fov_rad: F = <F as BaseFloat>::pi() * <F as NumCast>::from(self.fov).unwrap() /
                         Cast::from(180.0);
        let distance_from_screen_center: F =
            (screen_width * screen_width + screen_height * screen_height).sqrt() /
            (<F as NumCast>::from(2.0).unwrap() * (fov_rad / Cast::from(2.0)).tan());
        let screen_center_point_4d = self.location + self.forward * distance_from_screen_center;
        let screen_point_4d = screen_center_point_4d + (self.up * rel_y) + (right * rel_x);

        (screen_point_4d - self.location).normalize()
    }

    fn max_depth(&self) -> u32 {
        self.max_depth
    }

    fn update(&mut self, delta_time: &Duration, context: &SimulationContext, universe: &Universe4<F>) {
        let delta_millis = <F as NumCast>::from((*delta_time * 1000u32).as_secs()).unwrap() /
                           Cast::from(1000.0);

        self.update_rotation(delta_millis, context);

        let pressed_keys: &HashSet<VirtualKeyCode> = context.pressed_keys();
        let mut distance = self.speed * delta_millis;

        if distance == <F as Zero>::zero() {
            return;
        }

        let mut direction: Vector4<F> = na::zero();

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
                                             4_u32 * ApproxEq::<F>::approx_ulps(None as Option<F>)) {
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

impl<F: CustomFloat> Entity<F, Point4<F>, Vector4<F>> for FreeCamera4<F> {
    fn as_traceable_mut(&mut self) -> Option<&mut Traceable<F, Point4<F>, Vector4<F>>> {
        None
    }

    fn as_traceable(&self) -> Option<&Traceable4<F>> {
        None
    }
}

impl<F: CustomFloat> Locatable<F, Point4<F>, Vector4<F>> for FreeCamera4<F> {
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

impl<F: CustomFloat> Rotatable<F, Point4<F>, Vector4<F>> for FreeCamera4<F> {
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
