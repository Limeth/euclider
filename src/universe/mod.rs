pub mod d3;
pub mod entity;

use std;
use std::time::Duration;
use std::collections::HashMap;
use std::any::TypeId;
use std::borrow::Cow;
use na::BaseFloat;
use na::NumPoint;
use na::FloatPoint;
use na::PointAsVector;
use num::Float;
use glium::Surface as GliumSurface;
use glium::texture::Texture2d;
use glium::backend::Facade;
use glium::texture::ClientFormat;
use image::Rgb;
use image::Rgba;
use glium::BlitTarget;
use glium::texture::RawImage2d;
use glium::uniforms::MagnifySamplerFilter;
use scoped_threadpool::Pool;
use SimulationContext;
use universe::entity::*;
use util;

pub trait Universe<F: BaseFloat> where Self: Sync {
    type P: NumPoint<F>;
    // Generics hell I might need in the future:
    //
    // fn camera_mut<C>(&mut self) -> &mut C where C: Camera<Self::P, Self::V>;
    // fn camera<C>(&self) -> &C where C: Camera<Self::P, Self::V>;
    // fn set_camera<C>(&mut self, camera: C) where C: Camera<Self::P, Self::V>;
    // fn entities_mut<E>(&mut self) -> &mut Vec<Box<E>> where E: Entity<Self::P, Self::V>;
    // fn entities<E>(&self) -> &Vec<Box<E>> where E: Entity<Self::P, Self::V>;
    // fn set_entities<E>(&mut self, entities: Vec<Box<E>>) where E: Entity<Self::P, Self::V>;
    // fn intersections_mut<F, M, S>(&mut self) -> &mut HashMap<(TypeId, TypeId), F>
    //     where F: Fn(M, S) -> Option<F, Self::P>,
    //           M: Material<Self::P, Self::V>,
    //           S: Shape<Self::P, Self::V>;
    // fn intersections<F, M, S>(&self) -> &HashMap<(TypeId, TypeId), F>
    //     where F: Fn(M, S) -> Option<F, Self::P>,
    //           M: Material<Self::P, Self::V>,
    //           S: Shape<Self::P, Self::V>;
    // fn set_intersections<F, M, S>(&mut self, intersections: &mut HashMap<(TypeId, TypeId), F>)
    //     where F: Fn(M, S) -> Option<F, Self::P>,
    //           M: Material<Self::P, Self::V>,
    //           S: Shape<Self::P, Self::V>;
    /// FIXME: Temporary method, because there is currently no way to do this in nalgebra
    fn nalgebra_operations(&self) -> &NalgebraOperations<F, Self::P>;
    fn camera_mut(&mut self) -> &mut Camera<F, Self::P>;
    fn camera(&self) -> &Camera<F, Self::P>;
    fn set_camera(&mut self, camera: Box<Camera<F, Self::P>>);
    fn entities_mut(&mut self) -> &mut Vec<Box<Entity<F, Self::P>>>;
    fn entities(&self) -> &Vec<Box<Entity<F, Self::P>>>;
    fn set_entities(&mut self, entities: Vec<Box<Entity<F, Self::P>>>);
    /// Calculates the intersection of the shape (second) in the material (first)
    fn intersectors_mut(&mut self)
                         -> &mut HashMap<(TypeId, TypeId),
                                         fn(&Self::P,
                                            &<Self::P as PointAsVector>::Vector,
                                            &Material<F, Self::P>,
                                            &Shape<F, Self::P>)
                                            -> Option<Intersection<F, Self::P>>>;
    fn intersectors(&self)
                     -> &HashMap<(TypeId, TypeId),
                                 fn(&Self::P,
                                    &<Self::P as PointAsVector>::Vector,
                                    &Material<F, Self::P>,
                                    &Shape<F, Self::P>)
                                    -> Option<Intersection<F, Self::P>>>;
    fn set_intersectors(&mut self,
                         intersections: HashMap<(TypeId, TypeId),
                                                fn(&Self::P,
                                                   &<Self::P as PointAsVector>::Vector,
                                                   &Material<F, Self::P>,
                                                   &Shape<F, Self::P>)
                                                   -> Option<Intersection<F, Self::P>>>);
    /// Stores the behavior of a ray passing from the first material to the second
    fn transitions_mut(&mut self)
                         -> &mut HashMap<(TypeId, TypeId),
                                         fn(&Material<F, Self::P>,
                                            &Material<F, Self::P>,
                                            &TracingContext<F, Self::P>
                                            ) -> Rgba<u8>>;
    fn transitions(&self)
                         -> &HashMap<(TypeId, TypeId),
                                         fn(&Material<F, Self::P>,
                                            &Material<F, Self::P>,
                                            &TracingContext<F, Self::P>
                                            ) -> Rgba<u8>>;
    fn set_transitions(&mut self,
                         transitions: HashMap<(TypeId, TypeId),
                                         fn(&Material<F, Self::P>,
                                            &Material<F, Self::P>,
                                            &TracingContext<F, Self::P>
                                            ) -> Rgba<u8>>);

    fn trace(&self, time: &Duration, belongs_to: &Traceable<F, Self::P>, location: &Self::P, rotation: &<Self::P as PointAsVector>::Vector) -> Rgba<u8> {
        let material = belongs_to.material();
        let mut foreground: Option<Rgba<u8>> = None;
        let mut foreground_distance_squared: Option<F> = None;

        for other in self.entities() {
            let other_traceable = other.as_traceable();

            if other_traceable.is_none() {
                continue;
            }

            let other_traceable = other_traceable.unwrap();
            let shape = other_traceable.shape();
            let material_id = material.id();
            let shape_id = shape.id();
            let intersector = self.intersectors().get(&(material_id, shape_id));

            if intersector.is_none() {
                continue;
            }

            let intersector = intersector.unwrap();

            match intersector(location, rotation, material, shape) {
                Some(intersection) => {
                    let normal = shape.get_normal_at(&intersection.location);
                    let surface = other_traceable.surface();

                    if surface.is_none() {
                        continue; //TODO
                    }

                    let surface = surface.unwrap();

                    if foreground_distance_squared.is_none()
                        || foreground_distance_squared.unwrap() > intersection.distance_squared {
                        let context = TracingContext {
                            time: time,
                            origin_traceable: belongs_to,
                            intersection_traceable: other_traceable,
                            intersection: &intersection,
                            nalgebra_operations: self.nalgebra_operations(),
                            transitions: self.transitions(),
                            trace: &|time, traceable, location, direction| {
                                self.trace(time, traceable, location, direction)
                            },
                        };

                        // Avoid a stack overflow, where a ray intersects the same location
                        // repeatedly.
                        if intersection.distance_squared <= std::f64::EPSILON as F * 1.0 {
                            continue;
                        }

                        foreground = Some(surface.get_color(context));
                        foreground_distance_squared = Some(intersection.distance_squared);
                    }

                    // // FIXME just for testing
                    // use na::Point3;
                    // use na::Vector3;
                    // let location = unsafe { &*(location as *const _ as *const Point3<F>) }.clone();
                    // let normal = unsafe { &*(&normal as *const _ as *const Vector3<F>) }.clone();
                    // let rotation = unsafe { &*(rotation as *const _ as *const Vector3<F>) }.clone();
                    // // Calculate the angle using the cosine formula
                    // // |u*v| = ||u|| * ||v|| * cos(alpha)
                    // let angle = (rotation.dot(&normal).abs() / (na::distance(&na::origin(), &rotation.to_point()) * na::distance(&na::origin(), &normal.to_point()) as F)).acos();
                    // color.data[1] = (255.0 * (1.0 - angle / std::F::consts::FRAC_PI_2)) as u8;
                    // color.data[3] = 255u8;

                    // // println!("origin: [{}; {}; {}]; vector: <{}; {}; {}>", location.x, location.y, location.z, rotation.x, rotation.y, rotation.z);

                    // if foreground_distance_squared.is_none()
                    //     || foreground_distance_squared.unwrap() > intersection.distance_squared {
                    //     foreground_distance_squared = Some(intersection.distance_squared);
                    //     foreground = Some(color);
                    // }
                },
                None => (),
            }
        }

        foreground.unwrap_or(Rgba {
            data: [0u8, 0u8, 0u8, 0u8],
        })
    }

    fn trace_unknown(&self, time: &Duration, location: &Self::P, rotation: &<Self::P as PointAsVector>::Vector) -> Option<Rgb<u8>> {
        let mut belongs_to: Option<&Traceable<F, Self::P>> = None;

        for entity in self.entities() {
            let traceable = entity.as_traceable();

            if traceable.is_none() {
                continue;
            }

            let traceable: &Traceable<F, Self::P> = traceable.unwrap();
            let shape: &Shape<F, Self::P> = traceable.shape();

            if !shape.is_point_inside(location) {
                continue;
            }

            belongs_to = Some(&*traceable);
            break;
        }

        if belongs_to.is_some() {
            let background = Rgb { data: [255u8, 255u8, 255u8], };
            let foreground = self.trace(time, belongs_to.unwrap(), location, rotation);
            Some(util::overlay_color(background, foreground))
        } else {
            None
        }
    }

    fn trace_screen_point(&self,
                          time: &Duration,
                          screen_x: i32,
                          screen_y: i32,
                          screen_width: i32,
                          screen_height: i32)
                          -> Rgb<u8> {
        let camera = self.camera();
        let point = camera.get_ray_point(screen_x, screen_y, screen_width, screen_height);
        let vector = camera.get_ray_vector(screen_x, screen_y, screen_width, screen_height);

        // if (screen_x - screen_width / 2 == 0 && screen_y - screen_height / 2 == 0)
        //     || (screen_x == 0 && screen_y == 0) {
        //     use na::Point3;
        //     use na::Vector3;
        //     let point = unsafe { &*(&point as *const _ as *const Point3<F>) };
        //     let vector = unsafe { &*(&vector as *const _ as *const Vector3<F>) };
        //     println!("{}; {}:   <{}; {}; {}>", screen_x, screen_y, vector.x, vector.y, vector.z);
        // }

        match self.trace_unknown(time, &point, &vector) {
            Some(color) => color,
            None => {
                let checkerboard_size = 8;

                match (screen_x / checkerboard_size + screen_y / checkerboard_size) % 2 == 0 {
                    true => Rgb { data: [0u8, 0u8, 0u8] },
                    false => Rgb { data: [255u8, 0u8, 255u8] },
                }
            }
        }
    }

    fn render<F: Facade, S: GliumSurface>(&self,
                                     facade: &F,
                                     surface: &mut S,
                                     time: &Duration,
                                     context: &SimulationContext) {
        let (width, height) = surface.get_dimensions();
        // let mut buffer: DynamicImage = DynamicImage::new_rgb8(width, height);
        const COLOR_DIM: usize = 3;
        let buffer_width = width / context.resolution;
        let buffer_height = height / context.resolution;
        let mut data: Vec<u8> = vec!(0; (buffer_width * buffer_height) as usize * COLOR_DIM);
        let mut pool = Pool::new(4);

        // TODO: This loop takes a long time!
        pool.scoped(|scope| {
            for (index, chunk) in &mut data.chunks_mut(COLOR_DIM).enumerate() {
                scope.execute(move || {
                    let x = index as u32 % buffer_width;
                    let y = index as u32 / buffer_width;
                    let color = self.trace_screen_point(time,
                                                        x as i32,
                                                        y as i32,
                                                        buffer_width as i32,
                                                        buffer_height as i32);

                    for (i, result) in chunk.iter_mut().enumerate() {
                        *result = color.data[i];
                    }
                });
            }
        });

        let image = RawImage2d {
            data: Cow::Owned(data),
            width: buffer_width,
            height: buffer_height,
            format: ClientFormat::U8U8U8,
        };
        let texture = Texture2d::new(facade, image).unwrap();
        let image_surface = texture.as_surface();
        let blit_target = BlitTarget {
            left: 0,
            bottom: 0,
            width: width as i32,
            height: height as i32,
        };

        image_surface.blit_whole_color_to(surface, &blit_target, MagnifySamplerFilter::Nearest);
    }

    fn update(&mut self, delta_time: &Duration, context: &SimulationContext) {
        self.camera_mut().as_updatable_mut().map(|x| x.update(delta_time, context));

        for entity in self.entities_mut() {
            let updatable = entity.as_updatable_mut();

            if updatable.is_some() {
                updatable.unwrap().update(delta_time, context);
            }
        }
    }
}

pub trait NalgebraOperations<F: BaseFloat, P: NumPoint<F>> {
    fn to_point(&self, vector: &<P as PointAsVector>::Vector) -> P;
    fn dot(&self, first: &<P as PointAsVector>::Vector, second: &<P as PointAsVector>::Vector) -> F;
    fn angle_between(&self, first: &<P as PointAsVector>::Vector, second: &<P as PointAsVector>::Vector) -> F;
}
