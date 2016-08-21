use num::traits::NumCast;
use rand::StdRng;
use rand::Rand;
use num::One;
use na;
use na::Rotate;
use na::UnitQuaternion;
use na::Cast;
use na::Point3;
use na::Point4;
use na::Vector4;
use na::Norm;
use noise::perlin4;
use noise::Seed;
use palette;
use palette::Hsv;
use palette::RgbHue;
use util::CustomFloat;
use util::AngleBetween;
use util::Derank;
use universe::entity::surface::Surface;
use universe::entity::surface::ThresholdDirectionProvider;
use universe::entity::surface::UVFn;
use universe::entity::surface::SurfaceColorProvider;
use universe::entity::shape::TracingContext;

pub type Surface4<F> = Surface<F, Point4<F>, Vector4<F>>;

pub fn uv_derank<F: CustomFloat>(uvfn: Box<UVFn<F, Point3<F>>>) -> Box<UVFn<F, Point4<F>>> {
    Box::new(move |point: &Point4<F>| {
        uvfn(&point.derank())
    })
}
