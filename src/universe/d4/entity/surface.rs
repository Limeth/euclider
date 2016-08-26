use na::Point3;
use na::Point4;
use na::Vector4;
use util::CustomFloat;
use util::Derank;
use universe::entity::surface::Surface;
use universe::entity::surface::UVFn;

pub type Surface4<F> = Surface<F, Point4<F>, Vector4<F>>;

pub fn uv_derank<F: CustomFloat>(uvfn: Box<UVFn<F, Point3<F>>>) -> Box<UVFn<F, Point4<F>>> {
    Box::new(move |point: &Point4<F>| {
        uvfn(&point.derank())
    })
}
