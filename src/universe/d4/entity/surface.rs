use universe::d3::Point3;
use universe::d4::Point4;
use universe::d4::Vector4;
use util::CustomFloat;
use util::Derank;
use universe::entity::surface::Surface;
use universe::entity::surface::UVFn;

pub type Surface4 = Surface<Point4, Vector4>;

pub fn uv_derank(uvfn: Box<UVFn<Point3>>) -> Box<UVFn<Point4>> {
    Box::new(move |point: &Point4| {
        uvfn(&point.derank())
    })
}
