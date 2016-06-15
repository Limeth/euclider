pub mod d3;
pub mod entity;

use std::time::Duration;
use std::collections::HashMap;
use std::any::TypeId;
use std::borrow::Cow;
use na;
use na::Cast;
use na::BaseFloat;
use na::NumPoint;
use na::PointAsVector;
use num::traits::NumCast;
use glium::Surface as GliumSurface;
use glium::texture::Texture2d;
use glium::backend::Facade;
use glium::texture::ClientFormat;
use image;
use palette::Blend;
use palette::Rgb;
use palette::Rgba;
use glium::BlitTarget;
use glium::texture::RawImage2d;
use glium::uniforms::MagnifySamplerFilter;
use scoped_threadpool::Pool;
use SimulationContext;
use universe::entity::*;
use util;
use util::CustomFloat;
use util::Consts;

pub trait Universe<F: CustomFloat>
    where Self: Sync
{
    type P: NumPoint<F>;
    type O: NalgebraOperations<F, Self::P>;
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
    fn camera_mut(&mut self) -> &mut Camera<F, Self::P, Self::O>;
    fn camera(&self) -> &Camera<F, Self::P, Self::O>;
    fn set_camera(&mut self, camera: Box<Camera<F, Self::P, Self::O>>);
    fn entities_mut(&mut self) -> &mut Vec<Box<Entity<F, Self::P, Self::O>>>;
    fn entities(&self) -> &Vec<Box<Entity<F, Self::P, Self::O>>>;
    fn set_entities(&mut self, entities: Vec<Box<Entity<F, Self::P, Self::O>>>);
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
                                          &TracingContext<F, Self::P, Self::O>)
                                          -> Option<Rgba<F>>>;
    fn transitions(&self)
                   -> &HashMap<(TypeId, TypeId),
                               fn(&Material<F, Self::P>,
                                  &Material<F, Self::P>,
                                  &TracingContext<F, Self::P, Self::O>)
                                  -> Option<Rgba<F>>>;
    fn set_transitions(&mut self,
                       transitions: HashMap<(TypeId, TypeId),
                                            fn(&Material<F, Self::P>,
                                               &Material<F, Self::P>,
                                               &TracingContext<F, Self::P, Self::O>)
                                               -> Option<Rgba<F>>>);

    fn trace(&self,
             time: &Duration,
             max_depth: &u32,
             belongs_to: &Traceable<F, Self::P, Self::O>,
             location: &Self::P,
             rotation: &<Self::P as PointAsVector>::Vector)
             -> Option<Rgba<F>> {
        let material = belongs_to.material();
        let mut foreground: Option<Rgba<F>> = None;
        let mut foreground_distance_squared: Option<F> = None;

        if *max_depth <= 0 {
            return None;
        }

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
                    let exiting: bool;
                    let closer_normal: <Self::P as PointAsVector>::Vector;

                    if <Self::O as NalgebraOperations<F, Self::P>>::angle_between(&intersection.direction,
                                                                      &normal)
                            < <F as BaseFloat>::frac_pi_2() {
                        closer_normal = <Self::O as NalgebraOperations<F, Self::P>>::neg(&normal);
                        exiting = true;
                        continue; //The same behavior as the above TODO
                    } else {
                        closer_normal = <Self::O as NalgebraOperations<F, Self::P>>::clone(&normal);
                        exiting = false;
                    }

                    if foreground_distance_squared.is_none() ||
                       foreground_distance_squared.unwrap() > intersection.distance_squared {
                        let context = TracingContext {
                            time: time,
                            depth_remaining: max_depth,
                            origin_traceable: belongs_to,
                            intersection_traceable: other_traceable,
                            intersection: &intersection,
                            intersection_normal: &normal,
                            intersection_normal_closer: &closer_normal,
                            exiting: &exiting,
                            transitions: self.transitions(),
                            trace: &|time, traceable, location, direction| {
                                self.trace(time, &(*max_depth - 1), traceable, location, direction)
                            },
                        };

                        // Avoid a stack overflow, where a ray intersects the same location
                        // repeatedly.
                        if intersection.distance_squared <=
                           <F as Consts>::epsilon() * <F as NumCast>::from(1000.0).unwrap() {
                            continue;
                        }

                        foreground = Some(surface.get_color(context));
                        foreground_distance_squared = Some(intersection.distance_squared);
                    }
                }
                None => (),
            }
        }

        foreground.or(Some(Rgba::new(Cast::from(0.0),
                                     Cast::from(0.0),
                                     Cast::from(0.0),
                                     Cast::from(0.0))))
    }

    fn trace_first(&self,
                   time: &Duration,
                   max_depth: &u32,
                   belongs_to: &Traceable<F, Self::P, Self::O>,
                   location: &Self::P,
                   rotation: &<Self::P as PointAsVector>::Vector)
                   -> Rgba<F> {
        self.trace(time, max_depth, belongs_to, location, rotation)
            .expect("Couldn't send out a ray; None returned.")
    }

    fn trace_unknown(&self,
                     time: &Duration,
                     max_depth: &u32,
                     location: &Self::P,
                     rotation: &<Self::P as PointAsVector>::Vector)
                     -> Option<Rgb<F>> {
        let mut belongs_to: Option<&Traceable<F, Self::P, Self::O>> = None;

        for entity in self.entities() {
            let traceable = entity.as_traceable();

            if traceable.is_none() {
                continue;
            }

            let traceable: &Traceable<F, Self::P, Self::O> = traceable.unwrap();
            let shape: &Shape<F, Self::P> = traceable.shape();

            if !shape.is_point_inside(location) {
                continue;
            }

            belongs_to = Some(&*traceable);
            break;
        }

        if belongs_to.is_some() {
            let background = Rgba::from(
                                Rgb::new(
                                    Cast::from(1.0),
                                    Cast::from(1.0),
                                    Cast::from(1.0)
                                )
                             ).into_premultiplied();
            let foreground = self.trace_first(time,
                                              max_depth,
                                              belongs_to.unwrap(),
                                              location,
                                              rotation)
                                 .into_premultiplied();
            Some(
                Rgb::from_premultiplied(
                    foreground.over(background)
                )
            )
            // Some(util::overlay_color::<F>(background, foreground))
        } else {
            None
        }
    }

    fn trace_screen_point(&self,
                          time: &Duration,
                          max_depth: &u32,
                          screen_x: i32,
                          screen_y: i32,
                          screen_width: i32,
                          screen_height: i32)
                          -> Rgb<F> {
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

        match self.trace_unknown(time, max_depth, &point, &vector) {
            Some(color) => color,
            None => {
                let checkerboard_size = 8;

                match (screen_x / checkerboard_size + screen_y / checkerboard_size) % 2 == 0 {
                    true => Rgb::new(Cast::from(0.0), Cast::from(0.0), Cast::from(0.0)),
                    false => Rgb::new(Cast::from(1.0), Cast::from(0.0), Cast::from(1.0)),
                }
            }
        }
    }

    fn render<E: Facade, S: GliumSurface>(&self,
                                          facade: &E,
                                          surface: &mut S,
                                          time: &Duration,
                                          context: &SimulationContext) {
        let (width, height) = surface.get_dimensions();
        // let mut buffer: DynamicImage = DynamicImage::new_rgb8(width, height);
        const COLOR_DIM: usize = 3;
        let buffer_width = width / context.resolution;
        let buffer_height = height / context.resolution;
        let max_depth = self.camera().max_depth();
        let mut data: Vec<u8> = vec!(0; (buffer_width * buffer_height) as usize * COLOR_DIM);
        let mut pool = Pool::new(4);

        pool.scoped(|scope| {
            for (index, chunk) in &mut data.chunks_mut(COLOR_DIM).enumerate() {
                scope.execute(move || {
                    let x = index as u32 % buffer_width;
                    let y = index as u32 / buffer_width;
                    let color = self.trace_screen_point(time,
                                                        &max_depth,
                                                        x as i32,
                                                        y as i32,
                                                        buffer_width as i32,
                                                        buffer_height as i32);
                    let color = image::Rgb {
                        data: color.to_pixel(),
                    };

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

pub trait NalgebraOperations<F: CustomFloat, P: NumPoint<F>> {
    fn to_point(vector: &<P as PointAsVector>::Vector) -> P;
    fn dot(first: &<P as PointAsVector>::Vector,
           second: &<P as PointAsVector>::Vector)
           -> F;
    fn angle_between(first: &<P as PointAsVector>::Vector,
                     second: &<P as PointAsVector>::Vector)
                     -> F;
    fn neg(vector: &<P as PointAsVector>::Vector) -> <P as PointAsVector>::Vector;
    fn clone(vector: &<P as PointAsVector>::Vector) -> <P as PointAsVector>::Vector;
}
