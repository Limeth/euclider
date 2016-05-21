extern crate nalgebra as na;
extern crate image;
extern crate glium;
pub mod d3;
mod entity;

use std::time::Duration;
use self::na::*;
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
use universe::entity::camera::Camera;
use universe::entity::Entity;
use universe::entity::Locatable;
use universe::entity::Updatable;

pub trait Universe {
    type P: NumPoint<f32>;
    type V: NumVector<f32>;
    fn camera_mut(&mut self) -> &mut Camera;
    fn camera(&self) -> &Camera;
    fn set_camera(&mut self, camera: &Camera);
    fn entities_mut(&mut self) -> &mut Vec<Box<Entity<Self::P, Self::V>>>;
    fn entities(&self) -> &Vec<Box<Entity<Self::P, Self::V>>>;
    fn set_entities(&mut self, entities: Vec<Box<Entity<Self::P, Self::V>>>);
    fn trace(&self, location: &Point3<f32>, rotation: &Vector3<f32>) -> Rgb<u8>;

    fn trace_screen_point(&self, screen_x: i32, screen_y: i32, screen_width: i32, screen_height: i32) -> Rgb<u8> {
        let camera = self.camera();
        let vector = camera.get_ray_vector(screen_x, screen_y, screen_width, screen_height);

        self.trace(camera.location(), &vector)
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
        self.camera_mut().update(delta_time, context);
    }
}
