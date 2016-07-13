use std::any::Any;
use std::any::TypeId;
use num::traits::NumCast;
use num::Zero;
use rand::Rng;
use rand::Rand;
use na;
use na::Cast;
use na::Point3;
use na::Vector3;
use noise::perlin4;
use noise::Seed;
use palette;
use palette::Rgba;
use palette::Hsv;
use palette::RgbHue;
use util::CustomFloat;
use util::HasId;
use palette::Alpha;
use palette::Rgb;
use na::BaseFloat;
use universe::entity::surface::Surface;
use universe::entity::surface::ReflectionRatioProvider;
use universe::entity::surface::ReflectionDirectionProvider;
use universe::entity::shape::TracingContext;
use universe::d3::entity::AXIS_Z;

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

// TODO: Generalize
#[allow(unused_variables)]
pub fn reflection_ratio_uniform<F: CustomFloat>(ratio: F)
        -> Box<ReflectionRatioProvider<F, Point3<F>, Vector3<F>>> {
    Box::new(move |context: &TracingContext<F, Point3<F>, Vector3<F>>| {
        ratio
    })
}

// TODO: Generalize
pub fn reflection_direction_specular<F: CustomFloat>()
        -> Box<ReflectionDirectionProvider<F, Point3<F>, Vector3<F>>> {
    Box::new(move |context: &TracingContext<F, Point3<F>, Vector3<F>>| {
        // R = 2*(V dot N)*N - V
        let mut normal = context.intersection.normal;

        if na::angle_between(&context.intersection.direction, &normal) > BaseFloat::frac_pi_2() {
            normal = -normal;
        }

        normal * <F as NumCast>::from(-2.0).unwrap() *
        na::dot(&context.intersection.direction, &normal) + context.intersection.direction
    })
}

pub fn get_surface_color_test<F: CustomFloat>(context: &TracingContext<F, Point3<F>, Vector3<F>>)
                                              -> Rgba<F> {
    let mut normal = context.intersection.normal;

    if na::angle_between(&context.intersection.direction, &normal) > BaseFloat::frac_pi_2() {
        normal = -normal;
    }

    let angle: F = na::angle_between(&normal, &AXIS_Z());

    Alpha {
        color: Rgb::from(Hsv::new(RgbHue::from(<F as Zero>::zero()),
                                  <F as Zero>::zero(),
                                  <F as NumCast>::from(angle / <F as BaseFloat>::pi()).unwrap())),
        alpha: Cast::from(0.5),
    }
}
