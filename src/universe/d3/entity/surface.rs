use std::any::Any;
use std::any::TypeId;
use num::traits::NumCast;
use rand::StdRng;
use rand::Rng;
use rand::Rand;
use na::Cast;
use na::Point2;
use na::Point3;
use na::Vector3;
use na::Norm;
use noise::perlin4;
use noise::Seed;
use palette;
use palette::Rgba;
use palette::Hsv;
use palette::RgbHue;
use util::CustomFloat;
use util::CustomPoint;
use util::CustomVector;
use util::HasId;
use universe::entity::surface::Surface;
use universe::entity::surface::UVFn;
use universe::entity::surface::SurfaceColorProvider;
use universe::entity::shape::TracingContext;

pub type Surface3<F> = Surface<F, Point3<F>, Vector3<F>>;

pub struct PerlinSurface3<F: CustomFloat> {
    seed: Seed,
    size: F,
    speed: F,
}

impl<F: CustomFloat> HasId for PerlinSurface3<F> {
    fn id(&self) -> TypeId {
        Self::id_static()
    }

    fn as_any(&self) -> &Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut Any {
        self
    }
}

impl<F: CustomFloat> PerlinSurface3<F> {
    #[allow(dead_code)]
    pub fn new(seed: u32, size: F, speed: F) -> PerlinSurface3<F> {
        PerlinSurface3 {
            seed: Seed::new(seed),
            size: size,
            speed: speed,
        }
    }

    pub fn rand<R: Rng>(rng: &mut R, size: F, speed: F) -> PerlinSurface3<F> {
        PerlinSurface3 {
            seed: Seed::rand(rng),
            size: size,
            speed: speed,
        }
    }
}

impl<F: CustomFloat> Surface<F, Point3<F>, Vector3<F>> for PerlinSurface3<F> {
    fn get_color(&self, context: TracingContext<F, Point3<F>, Vector3<F>>) -> Rgba<F> {
        let time_millis: F = Cast::from((*context.time * 1000).as_secs() as f64 / 1000.0);
        let location = [context.intersection.location.x / self.size,
                        context.intersection.location.y / self.size,
                        context.intersection.location.z / self.size,
                        time_millis * self.speed];
        let value = perlin4(&self.seed, &location);
        palette::Rgba::from(Hsv::new(RgbHue::from(value * Cast::from(360.0)),
                                     Cast::from(1.0),
                                     Cast::from(1.0)))
    }
}

pub fn surface_color_perlin_hue<F: CustomFloat>
    (seed: Seed, size: F, speed: F)
     -> Box<SurfaceColorProvider<F, Point3<F>, Vector3<F>>> {
    Box::new(move |context: &TracingContext<F, Point3<F>, Vector3<F>>| {
        let time_millis: F = Cast::from((*context.time * 1000).as_secs() as f64 / 1000.0);
        let location = [context.intersection.location.x / size,
                        context.intersection.location.y / size,
                        context.intersection.location.z / size,
                        time_millis * speed];
        let value = perlin4(&seed, &location);
        palette::Rgba::from(Hsv::new(RgbHue::from(value * Cast::from(360.0)),
                                     Cast::from(1.0),
                                     Cast::from(1.0)))
    })
}

pub fn surface_color_perlin_hue_seed<F: CustomFloat>
    (seed: u32, size: F, speed: F)
     -> Box<SurfaceColorProvider<F, Point3<F>, Vector3<F>>> {
     surface_color_perlin_hue(Seed::new(seed), size, speed)
}

pub fn surface_color_perlin_hue_random<F: CustomFloat>
    (size: F, speed: F)
     -> Box<SurfaceColorProvider<F, Point3<F>, Vector3<F>>> {
     surface_color_perlin_hue(Seed::rand(&mut StdRng::new().expect("Could not create a random number generator.")),
                              size, speed)
}

pub fn uv_sphere<F: CustomFloat>(center_location: Point3<F>) -> Box<UVFn<F, Point3<F>>> {
    Box::new(move |point: &Point3<F>| {
        let point = *point - center_location;
        let point = point.normalize();
        Point2::new(<F as NumCast>::from(0.5).unwrap() +
                    point.y.atan2(point.x) / (<F as NumCast>::from(2.0).unwrap() * F::pi()),
                    <F as NumCast>::from(0.5).unwrap() - point.z.asin() / F::pi())
    })
}
