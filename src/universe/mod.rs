extern crate nalgebra as na;
extern crate image;
extern crate glium;
pub mod d3;
mod camera;

use std::time::Duration;
use self::na::*;
use self::glium::Surface;
use self::glium::texture::SrgbTexture2d;
use self::glium::texture::Texture2d;
use self::glium::backend::Facade;
use self::image::Rgb;
use self::image::Rgba;
use self::image::Pixel;
use self::image::DynamicImage;
use self::image::GenericImage;
use self::glium::BlitTarget;
use self::glium::texture::RawImage2d;
use self::glium::texture::Texture2dDataSource;
use self::glium::texture::PixelValue;
use self::glium::uniforms::MagnifySamplerFilter;
use ::SimulationContext;
use universe::camera::Camera;

pub trait Universe {
    fn camera_mut(&mut self) -> &mut Camera;
    fn camera(&self) -> &Camera;
    fn set_camera(&mut self, camera: &Camera);
    fn entities_mut(&mut self) -> &mut Vec<Box<Entity>>;
    fn entities(&self) -> &Vec<Box<Entity>>;
    fn set_entities(&mut self, entities: Vec<Box<Entity>>);

    fn trace(&self, screen_x: i32, screen_y: i32) -> Rgb<u8> {
        Rgb {
            data: [
                (self.camera().rotation().x * 255.0) as u8,
                (self.camera().rotation().y * 255.0) as u8,
                (self.camera().rotation().z * 255.0) as u8
            ],
        }
    }

    fn render<F: Facade, S: Surface>(&self, facade: &F, surface: &mut S, time: &Duration, context: &SimulationContext) {
        let (width, height) = surface.get_dimensions();
        let mut buffer: DynamicImage = DynamicImage::new_rgb8(width, height);

        for x in 0 .. width {
            for y in 0 .. height {
                buffer.put_pixel(x, y, self.trace(x as i32, y as i32).to_rgba())
            }
        }

        // Convert into an OpenGL texture and draw the result
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

pub trait Entity {
    fn as_updatable(&mut self) -> Option<&mut Updatable>;
    // fn as_drawable<T: Drawable>(&mut self) -> &mut T;
    fn as_traceable(&mut self) -> Option<&mut Traceable>;
}

pub trait Updatable {
    fn update(&mut self, delta_time: &Duration, context: &SimulationContext);
}

// pub trait Drawable {
//     fn render<S: Surface>(&self, surface: &mut S, time: &Duration, context: &SimulationContext);
// }

pub trait Traceable {
    fn trace(&self) -> Rgba<u8>;
}

pub trait Locatable<P: NumPoint<f32>> {
    fn location_mut(&mut self) -> &mut P;
    fn location(&self) -> &P;
    fn set_location(&mut self, location: P);
}

pub trait Rotatable<P: NumVector<f32>> {
    fn rotation_mut(&mut self) -> &mut P;
    fn rotation(&self) -> &P;
    fn set_rotation(&mut self, location: P);
}
