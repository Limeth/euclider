extern crate nalgebra as na;
extern crate image;
extern crate glium;
pub mod d3;
pub mod entity;

use std::time::Duration;
use std::collections::HashMap;
use std::any::TypeId;
use self::na::Point3;
use self::na::Vector3;
use self::na::NumPoint;
use self::na::NumVector;
use self::glium::Surface;
use self::glium::texture::Texture2d;
use self::glium::backend::Facade;
use self::image::Rgb;
use self::image::Pixel;
use self::image::DynamicImage;
use self::image::GenericImage;
use self::glium::BlitTarget;
use self::glium::texture::RawImage2d;
use self::glium::uniforms::MagnifySamplerFilter;
use SimulationContext;
use universe::entity::Material;
use universe::entity::Camera;
use universe::entity::Entity;
use universe::entity::Locatable;
use universe::entity::Updatable;
use universe::entity::Traceable;
use universe::entity::Shape;

pub trait Universe {
    type P: NumPoint<f32>;
    type V: NumVector<f32>;
    // Generics hell I might need in the future:
    //
    // fn camera_mut<C>(&mut self) -> &mut C where C: Camera<Self::P, Self::V>;
    // fn camera<C>(&self) -> &C where C: Camera<Self::P, Self::V>;
    // fn set_camera<C>(&mut self, camera: C) where C: Camera<Self::P, Self::V>;
    // fn entities_mut<E>(&mut self) -> &mut Vec<Box<E>> where E: Entity<Self::P, Self::V>;
    // fn entities<E>(&self) -> &Vec<Box<E>> where E: Entity<Self::P, Self::V>;
    // fn set_entities<E>(&mut self, entities: Vec<Box<E>>) where E: Entity<Self::P, Self::V>;
    // fn intersections_mut<F, M, S>(&mut self) -> &mut HashMap<(TypeId, TypeId), F>
    //     where F: Fn(M, S) -> Option<Self::P>,
    //           M: Material<Self::P, Self::V>,
    //           S: Shape<Self::P, Self::V>;
    // fn intersections<F, M, S>(&self) -> &HashMap<(TypeId, TypeId), F>
    //     where F: Fn(M, S) -> Option<Self::P>,
    //           M: Material<Self::P, Self::V>,
    //           S: Shape<Self::P, Self::V>;
    // fn set_intersections<F, M, S>(&mut self, intersections: &mut HashMap<(TypeId, TypeId), F>)
    //     where F: Fn(M, S) -> Option<Self::P>,
    //           M: Material<Self::P, Self::V>,
    //           S: Shape<Self::P, Self::V>;
    fn camera_mut(&mut self) -> &mut Camera<Self::P, Self::V>;
    fn camera(&self) -> &Camera<Self::P, Self::V>;
    fn set_camera(&mut self, camera: Box<Camera<Self::P, Self::V>>);
    fn entities_mut(&mut self) -> &mut Vec<Box<Entity<Self::P, Self::V>>>;
    fn entities(&self) -> &Vec<Box<Entity<Self::P, Self::V>>>;
    fn set_entities(&mut self, entities: Vec<Box<Entity<Self::P, Self::V>>>);
    fn intersections_mut(&mut self) -> &mut HashMap<(TypeId, TypeId), &'static Fn(Material<Self::P, Self::V>, Shape<Self::P, Self::V>) -> Option<Self::P>>;
    fn intersections(&self) -> &HashMap<(TypeId, TypeId), &'static Fn(Material<Self::P, Self::V>, Shape<Self::P, Self::V>) -> Option<Self::P>>;
    fn set_intersections(&mut self, intersections: HashMap<(TypeId, TypeId), &'static Fn(Material<Self::P, Self::V>, Shape<Self::P, Self::V>) -> Option<Self::P>>);

    fn trace(&self, location: &Self::P, rotation: &Self::V) -> Option<Rgb<u8>> {
        let mut belongs_to: Option<&Entity<Self::P, Self::V>> = None;

        for entity in self.entities() {
            let traceable = entity.as_traceable();

            if traceable.is_none() {
                continue;
            }

            let traceable: &Traceable<Self::P, Self::V> = traceable.unwrap();
            let shape: &Shape<Self::P, Self::V> = traceable.shape();

            if !shape.is_point_inside(location) {
                continue;
            }

            belongs_to = Some(&**entity);
            break;
        }

        if belongs_to.is_some() {
            // TODO Trace from within the entity
        }

        None
    }

    fn trace_screen_point(&self, screen_x: i32, screen_y: i32, screen_width: i32, screen_height: i32) -> Rgb<u8> {
        let camera = self.camera();
        let point = camera.get_ray_point(screen_x, screen_y, screen_width, screen_height);
        let vector = camera.get_ray_vector(screen_x, screen_y, screen_width, screen_height);

        match self.trace(&point, &vector) {
            Some(color) => color,
            None => {
                let checkerboard_size = 8;

                match (screen_x / checkerboard_size + screen_y / checkerboard_size) % 2 == 0 {
                    true => Rgb {
                        data: [0u8, 0u8, 0u8],
                    },
                    false => Rgb {
                        data: [255u8, 0u8, 255u8],
                    },
                }
            }
        }
    }

    fn render<F: Facade, S: Surface>(&self, facade: &F, surface: &mut S, time: &Duration, context: &SimulationContext) {
        let (width, height) = surface.get_dimensions();
        let mut buffer: DynamicImage = DynamicImage::new_rgb8(width, height);

        // TODO: This loop takes a long time!
        for x in 0 .. width {
            for y in 0 .. height {
                buffer.put_pixel(x, y, self.trace_screen_point(x as i32, y as i32, width as i32, height as i32).to_rgba())
            }
        }

        // Convert into an OpenGL texture and draw the result
        // TODO: This next method takes a long time!
        let image = RawImage2d::from_raw_rgb_reversed(buffer.raw_pixels(), buffer.dimensions());
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
