use ::F;
use universe::d3::Point3;
use universe::d3::Vector3;
use num::traits::NumCast;
use rand;
use na;
use na::Cast;
use na::Norm;
use na::BaseFloat;
use noise::{Perlin, NoiseModule, Seedable};
use palette;
use palette::Hsv;
use palette::RgbHue;
use util::CustomFloat;
use universe::entity::surface::Surface;
use universe::entity::surface::UVFn;
use universe::entity::surface::SurfaceColorProvider;
use universe::entity::shape::TracingContext;

pub type Surface3 = Surface<Point3, Vector3>;

pub fn surface_color_perlin_hue
    (seed: usize,
     size: F,
     speed: F)
     -> Box<SurfaceColorProvider<Point3, Vector3>> {
    let perlin = Perlin::new();

    perlin.set_seed(seed);

    Box::new(move |context: &TracingContext<Point3, Vector3>| {
        let time_millis: F = Cast::from((context.time * 1000).as_secs() as f64 / 1000.0);
        let location = [context.intersection.location.x / size,
                        context.intersection.location.y / size,
                        context.intersection.location.z / size,
                        time_millis * speed];
        let value = perlin.get(location);
        palette::Rgba::<F>::from(Hsv::new(RgbHue::from(value * 360.0), 1.0, 1.0))
    })
}

// TODO required?
pub fn surface_color_perlin_hue_seed
    (seed: u32,
     size: F,
     speed: F)
     -> Box<SurfaceColorProvider<Point3, Vector3>> {
    surface_color_perlin_hue(seed as usize, size, speed)
}

pub fn surface_color_perlin_hue_random
    (size: F,
     speed: F)
     -> Box<SurfaceColorProvider<Point3, Vector3>> {
    surface_color_perlin_hue(rand::random(),
                             size,
                             speed)
}

pub fn uv_sphere(center_location: Point3) -> Box<UVFn<Point3>> {
    Box::new(move |point: &Point3| {
        let point = *point - center_location;
        let point = point.normalize();
        na::Point2::<F>::new(<F as NumCast>::from(0.5).unwrap() +
                    point.y.atan2(point.x) / (<F as NumCast>::from(2.0).unwrap() * <F as BaseFloat>::pi()),
                    <F as NumCast>::from(0.5).unwrap() - point.z.asin() / <F as BaseFloat>::pi())
    })
}
