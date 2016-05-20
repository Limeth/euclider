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
use self::image::Rgba;
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

    fn render<F: Facade, S: Surface>(&self, facade: &F, surface: &mut S, time: &Duration, context: &SimulationContext) {
        let (width, height) = surface.get_dimensions();
        surface.clear_color(0.0, 0.0, 1.0, 1.0);

        // let buffer: Vec<Vec<Rgba<u8>>> = Vec::new();
        // buffer.into_raw();
        let mut buffer: DynamicImage = DynamicImage::new_rgba8(width, height);
        buffer.put_pixel(0, 0, self.camera().trace());
        // let texture: SrgbTexture2d = SrgbTexture2d::new(facade, buffer);
        // let texture = Texture2d::new(facade, buffer);
        let image = RawImage2d::from_raw_rgba_reversed(buffer.raw_pixels(), buffer.dimensions());
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
    fn get_location(&self) -> P;
    fn set_location(&mut self, location: P);
}

pub trait Rotatable<P: NumVector<f32>> {
    fn get_rotation(&self) -> P;
    fn set_rotation(&mut self, location: P);
}
