#[macro_use]
pub mod entity;
pub mod d3;

use std::time::Duration;
use std::borrow::Cow;
use std::marker::Reflect;
use na::Cast;
use na::BaseFloat;
use glium::texture::ClientFormat;
use image;
use palette::Blend;
use palette::Rgb;
use palette::Rgba;
use glium::texture::RawImage2d;
use scoped_threadpool::Pool;
use simulation::SimulationContext;
use universe::entity::Entity;
use universe::entity::Camera;
use universe::entity::Traceable;
use universe::entity::material::Material;
use universe::entity::shape::Shape;
use universe::entity::shape::GeneralIntersectors;
use universe::entity::shape::Intersection;
use universe::entity::shape::Intersector;
use universe::entity::shape::TracingContext;
use universe::entity::shape::ColorTracingContext;
use universe::entity::shape::PathTracingContext;
use universe::entity::surface::MappedTexture;
use util::CustomPoint;
use util::CustomVector;
use util::CustomFloat;
use util::VectorAsPoint;
use util::AngleBetween;
use util::Provider;

pub type TraceResult<'a, F, P, V> = (&'a Traceable<F, P, V>,
                                     TracingContext<'a, F, P, V>);

pub trait Universe<F: CustomFloat>
    where Self: Sync + Reflect + 'static
{
    type P: CustomPoint<F, Self::V>;
    type V: CustomVector<F, Self::P>;

    fn camera_mut(&mut self) -> &mut Camera<F, Self::P, Self::V>;
    fn camera(&self) -> &Camera<F, Self::P, Self::V>;
    fn set_camera(&mut self, camera: Box<Camera<F, Self::P, Self::V>>);
    fn entities_mut(&mut self) -> &mut Vec<Box<Entity<F, Self::P, Self::V>>>;
    fn entities(&self) -> &Vec<Box<Entity<F, Self::P, Self::V>>>;
    fn set_entities(&mut self, entities: Vec<Box<Entity<F, Self::P, Self::V>>>);
    /// Calculates the intersection of the shape (second) in the material (first)
    fn intersectors_mut(&mut self) -> &mut GeneralIntersectors<F, Self::P, Self::V>;
    fn intersectors(&self) -> &GeneralIntersectors<F, Self::P, Self::V>;
    fn set_intersectors(&mut self, intersections: GeneralIntersectors<F, Self::P, Self::V>);
    fn background_mut(&mut self) -> &mut Box<MappedTexture<F, Self::P, Self::V>>;
    fn background(&self) -> &Box<MappedTexture<F, Self::P, Self::V>>;
    fn set_background(&mut self, background: Box<MappedTexture<F, Self::P, Self::V>>);

    fn intersect(&self,
                 location: &Self::P,
                 direction: &Self::V,
                 material: &Material<F, Self::P, Self::V>,
                 shape: &Shape<F, Self::P, Self::V>)
                 -> Provider<Intersection<F, Self::P, Self::V>> {
        let material_id = material.id();
        let shape_id = shape.id();
        let intersector = self.intersectors().get(&(material_id, shape_id));

        let intersector = intersector.expect(&format!("Couldn't find an intersector for material {} and shape {}.",
                                             material, shape));
        // if intersector.is_none() {
        //     continue;
        // }

        // let intersector = intersector.unwrap();

        let intersect: Intersector<F, Self::P, Self::V> =
            &move |material, shape| self.intersect(location, direction, material, shape);

        Provider::new(intersector(location, direction, material, shape, intersect))
    }

    fn trace_closest<'a>(&'a self,
                         time: &Duration,
                         belongs_to: &'a Traceable<F, Self::P, Self::V>,
                         location: &Self::P,
                         direction: &Self::V,
                         filter: &Fn(&Traceable<F, Self::P, Self::V>) -> bool)
                         -> Option<TraceResult<'a, F, Self::P, Self::V>> {
        let material = belongs_to.material();
        let mut closest: Option<TraceResult<'a, F, Self::P, Self::V>> = None;
        let mut closest_distance_squared: Option<F> = None;

        for other in self.entities() {
            let other_traceable = other.as_traceable();

            if other_traceable.is_none() {
                continue;
            }

            let other_traceable = other_traceable.unwrap();

            if !filter(other_traceable) {
                continue;
            }

            let shape = other_traceable.shape();
            let provider = self.intersect(location, direction, material, shape);
            let mut intersections = provider.iter();

            if let Some(intersection) = intersections.next() {
                let exiting: bool;
                let closer_normal: Self::V;

                // TODO
                if intersection.direction.angle_between(&intersection.normal) <
                   <F as BaseFloat>::frac_pi_2() {
                    // closer_normal = -intersection.normal;
                    // exiting = true;
                    // TODO NEXT ITERATION FROM THE INTERSECTIONS
                    continue; //The same behavior as the above TODO
                } else {
                    closer_normal = intersection.normal;
                    exiting = false;
                }

                if closest_distance_squared.is_none() ||
                   closest_distance_squared.unwrap() > intersection.distance_squared {
                    let context = TracingContext {
                        time: *time,
                        origin_traceable: belongs_to,
                        origin_location: *location,
                        origin_direction: *direction,
                        intersection_traceable: other_traceable,
                        intersection: *intersection,
                        intersection_normal_closer: closer_normal,
                        exiting: exiting,
                    };
                    closest = Some((other_traceable, context));
                    closest_distance_squared = Some(intersection.distance_squared);
                }
            }
        }

        closest
    }

    fn trace(&self,
             time: &Duration,
             max_depth: &u32,
             belongs_to: &Traceable<F, Self::P, Self::V>,
             location: &Self::P,
             direction: &Self::V)
             -> Rgba<F> {
        if *max_depth > 0 {
            let result = self.trace_closest(time, belongs_to, location, direction, &|other| {
                other.surface().is_some()
            });
            
            if result.is_some() {
                let (closest, general_context) = result.unwrap();

                let context = ColorTracingContext {
                    general: general_context,
                    depth_remaining: max_depth,
                    trace: &|time, traceable, location, direction| {
                        self.trace(time, &(*max_depth - 1), traceable, location, direction)
                    },
                };

                // We can safely unwrap here, because we filtered out all the entities without a surface.
                let surface = closest.surface().unwrap();

                return surface.get_color(context);
            }
        }

        self.background().get_color(direction.as_point())
    }

    fn trace_path(&self,
             time: &Duration,
             distance: &F,
             belongs_to: &Traceable<F, Self::P, Self::V>,
             location: &Self::P,
             direction: &Self::V)
             -> (Self::P, Self::V) {
        let result = self.trace_closest(time, belongs_to, location, direction, &|other| {
            other.surface().is_some()
        });
        
        if result.is_some() {
            let (closest, general_context) = result.unwrap();

            let context = PathTracingContext {
                general: general_context,
                distance: distance,
                trace: &|time, distance, traceable, location, direction| {
                    self.trace_path(time, distance, traceable, location, direction)
                },
            };

            // We can safely unwrap here, because we filtered out all the entities without a surface.
            let surface = closest.surface().unwrap();
            let path = surface.get_path(context);

            if path.is_some() {
                return path.unwrap();
            }
        }

        belongs_to.material().trace_path(location, direction, distance)
    }

    fn material_at(&self, location: &Self::P) -> Option<&Traceable<F, Self::P, Self::V>> {
        let mut belongs_to: Option<&Traceable<F, Self::P, Self::V>> = None;

        for entity in self.entities() {
            let traceable = entity.as_traceable();

            if traceable.is_none() {
                continue;
            }

            let traceable: &Traceable<F, Self::P, Self::V> = traceable.unwrap();
            let shape: &Shape<F, Self::P, Self::V> = traceable.shape();

            if !shape.is_point_inside(location) {
                continue;
            }

            belongs_to = Some(&*traceable);
            break;
        }

        belongs_to
    }

    fn trace_unknown(&self,
                     time: &Duration,
                     max_depth: &u32,
                     location: &Self::P,
                     direction: &Self::V)
                     -> Option<Rgb<F>> {
        self.material_at(location).map(|belongs_to| {
            let mut transitioned_direction = *direction;
            belongs_to.material().enter(location, &mut transitioned_direction);
            let background =
                Rgba::from(Rgb::new(Cast::from(1.0), Cast::from(1.0), Cast::from(1.0)))
                    .into_premultiplied();
            let foreground = self.trace(time, max_depth, belongs_to, location, &transitioned_direction)
                .into_premultiplied();
            Rgb::from_premultiplied(foreground.over(background))
        })
    }

    fn trace_path_unknown(&self,
                          time: &Duration,
                          distance: &F,
                          location: &Self::P,
                          direction: &Self::V)
                          -> Option<(Self::P, Self::V)> {
        self.material_at(location).map(|belongs_to| {
            let mut transitioned_direction = *direction;

            belongs_to.material().enter(location, &mut transitioned_direction);
            self.trace_path(time, distance, belongs_to, location, &transitioned_direction)
        })
    }
}

pub trait Environment<F: CustomFloat>: Sync {
    fn max_depth(&self) -> u32;
    fn trace_screen_point(&self,
                          time: &Duration,
                          max_depth: &u32,
                          screen_x: i32,
                          screen_y: i32,
                          screen_width: i32,
                          screen_height: i32)
                          -> Rgb<F>;
    fn render(&self,
              dimensions: (u32, u32),
              time: &Duration,
              context: &SimulationContext)
              -> RawImage2d<u8> {
        let (width, height) = dimensions;
        const COLOR_DIM: usize = 3;
        let buffer_width = width / context.resolution;
        let buffer_height = height / context.resolution;
        let max_depth = self.max_depth();
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
                    let color = image::Rgb { data: color.to_pixel() };

                    for (i, result) in chunk.iter_mut().enumerate() {
                        *result = color.data[i];
                    }
                });
            }
        });

        RawImage2d {
            data: Cow::Owned(data),
            width: buffer_width,
            height: buffer_height,
            format: ClientFormat::U8U8U8,
        }
    }

    fn update(&mut self, delta_time: &Duration, context: &SimulationContext);
}

impl<F: CustomFloat, P: CustomPoint<F, V>, V: CustomVector<F, P>, U: Universe<F, P=P, V=V>>
        Environment<F> for U {
    fn max_depth(&self) -> u32 {
        self.camera().max_depth()
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

        match self.trace_unknown(time, max_depth, &point, &vector) {
            Some(color) => color,
            None => {
                let checkerboard_size = 8;

                if (screen_x / checkerboard_size + screen_y / checkerboard_size) % 2 == 0 {
                    Rgb::new(Cast::from(0.0), Cast::from(0.0), Cast::from(0.0))
                } else {
                    Rgb::new(Cast::from(1.0), Cast::from(0.0), Cast::from(1.0))
                }
            }
        }
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
