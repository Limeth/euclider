use na::Point3;
use na::Vector3;
use palette::Rgba;
use util::CustomFloat;
use universe::entity::material::Material;
use universe::entity::shape::TracingContext;

pub type Material3<F> = Material<F, Point3<F>, Vector3<F>>;

// TODO: Generalize it?
#[allow(unused_variables)]
pub fn transition_vacuum_vacuum<F: CustomFloat>(from: &Material<F, Point3<F>, Vector3<F>>,
                                                to: &Material<F, Point3<F>, Vector3<F>>,
                                                context: &TracingContext<F, Point3<F>, Vector3<F>>)
                                                -> Option<Rgba<F>> {
    let trace = context.trace;
    trace(context.time,
          context.intersection_traceable,
          &context.intersection.location,
          &context.intersection.direction)
}
