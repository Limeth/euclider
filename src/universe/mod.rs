pub mod d3;
pub mod entity;

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
use universe::entity::material::TransitionHandlers;
use universe::entity::shape::Shape;
use universe::entity::shape::GeneralIntersectors;
use universe::entity::shape::Intersection;
use universe::entity::shape::Intersector;
use universe::entity::shape::TracingContext;
use universe::entity::surface::MappedTexture;
use util::CustomPoint;
use util::CustomVector;
use util::CustomFloat;
use util::VectorAsPoint;
use util::AngleBetween;
use util::Provider;

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
    /// Stores the behavior of a ray passing from the first material to the second
    fn transitions_mut(&mut self) -> &mut TransitionHandlers<F, Self::P, Self::V>;
    fn transitions(&self) -> &TransitionHandlers<F, Self::P, Self::V>;
    fn set_transitions(&mut self, transitions: TransitionHandlers<F, Self::P, Self::V>);
    fn background_mut(&mut self) -> &mut Box<MappedTexture<F, Self::P, Self::V>>;
    fn background(&self) -> &Box<MappedTexture<F, Self::P, Self::V>>;
    fn set_background(&mut self, background: Box<MappedTexture<F, Self::P, Self::V>>);

    fn intersect(&self,
                 location: &Self::P,
                 rotation: &Self::V,
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
            &move |material, shape| self.intersect(location, rotation, material, shape);

        Provider::new(intersector(location, rotation, material, shape, intersect))
    }

    fn trace(&self,
             time: &Duration,
             max_depth: &u32,
             belongs_to: &Traceable<F, Self::P, Self::V>,
             location: &Self::P,
             rotation: &Self::V)
             -> Rgba<F> {
        let material = belongs_to.material();
        let mut foreground: Option<Rgba<F>> = None;
        let mut foreground_distance_squared: Option<F> = None;

        if *max_depth == 0 {
            return self.background().get_color(rotation.as_point());
        }

        for other in self.entities() {
            let other_traceable = other.as_traceable();

            if other_traceable.is_none() {
                continue;
            }

            let other_traceable = other_traceable.unwrap();
            let shape = other_traceable.shape();
            let provider = self.intersect(location, rotation, material, shape);
            let mut intersections = provider.iter();

            if let Some(intersection) = intersections.next() {
                let surface = other_traceable.surface();

                if surface.is_none() {
                    continue; //TODO
                }

                let surface = surface.unwrap();
                let exiting: bool;
                let closer_normal: Self::V;

                // TODO
                if intersection.direction.angle_between(&intersection.normal) <
                   <F as BaseFloat>::frac_pi_2() {
                    // closer_normal = -intersection.normal;
                    // exiting = true;
                    continue; //The same behavior as the above TODO
                } else {
                    closer_normal = intersection.normal;
                    exiting = false;
                }

                if foreground_distance_squared.is_none() ||
                   foreground_distance_squared.unwrap() > intersection.distance_squared {
                    let context = TracingContext {
                        time: time,
                        depth_remaining: max_depth,
                        origin_traceable: belongs_to,
                        intersection_traceable: other_traceable,
                        intersection: intersection,
                        intersection_normal_closer: &closer_normal,
                        exiting: &exiting,
                        transitions: self.transitions(),
                        trace: &|time, traceable, location, direction| {
                            self.trace(time, &(*max_depth - 1), traceable, location, direction)
                        },
                    };

                    foreground = Some(surface.get_color(context));
                    foreground_distance_squared = Some(intersection.distance_squared);
                }
            }
        }

        foreground.unwrap_or_else(|| self.background().get_color(rotation.as_point()))
    }

    fn trace_unknown(&self,
                     time: &Duration,
                     max_depth: &u32,
                     location: &Self::P,
                     rotation: &Self::V)
                     -> Option<Rgb<F>> {
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

        if belongs_to.is_some() {
            let background =
                Rgba::from(Rgb::new(Cast::from(1.0), Cast::from(1.0), Cast::from(1.0)))
                    .into_premultiplied();
            let foreground = self.trace(time, max_depth, belongs_to.unwrap(), location, rotation)
                .into_premultiplied();
            Some(Rgb::from_premultiplied(foreground.over(background)))
            // Some(util::overlay_color::<F>(background, foreground))
        } else {
            None
        }
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
              context: &SimulationContext) -> RawImage2d<u8> {
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
