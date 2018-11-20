use ::F;
use universe::d3::Point3;
use universe::d3::Vector3;
use num::traits::NumCast;
use num::Zero;
use num::One;
use na;
use util::CustomFloat;
use universe::entity::shape::Shape;
use universe::entity::shape::Hyperplane;
use universe::entity::shape::HalfSpace;
use universe::entity::shape::ComposableShape;
use universe::entity::shape::SetOperation;

pub type Shape3 = Shape<Point3, Vector3>;

pub fn cuboid(center: Point3, abc: Vector3) -> ComposableShape<Point3, Vector3> {
    let half_abc: Vector3 = abc / <F as NumCast>::from(2.0).unwrap();
    let x = Vector3::new(<F as One>::one(), <F as Zero>::zero(), <F as Zero>::zero());
    let y = Vector3::new(<F as Zero>::zero(), <F as One>::one(), <F as Zero>::zero());
    let z = Vector3::new(<F as Zero>::zero(), <F as Zero>::zero(), <F as One>::one());
    let shapes: Vec<Box<Shape<Point3, Vector3>>> = vec![Box::new(HalfSpace::new_with_point(Hyperplane::new_with_vectors(&y,
                                                                             &z,
                                                                             &na::translate(&(x *
                                                                                 half_abc),
                                                                               &center),
                                                    ),
                                                    &center)),
                             Box::new(HalfSpace::new_with_point(Hyperplane::new_with_vectors(&y,
                                                                             &z,
                                                                             &na::translate(&-(x *
                                                                                  half_abc),
                                                                               &center),
                                                    ),
                                                    &center)),
                             Box::new(HalfSpace::new_with_point(Hyperplane::new_with_vectors(&x,
                                                                             &z,
                                                                             &na::translate(&(y *
                                                                                 half_abc),
                                                                               &center),
                                                    ),
                                                    &center)),
                             Box::new(HalfSpace::new_with_point(Hyperplane::new_with_vectors(&x,
                                                                             &z,
                                                                             &na::translate(&-(y *
                                                                                  half_abc),
                                                                               &center),
                                                    ),
                                                    &center)),
                             Box::new(HalfSpace::new_with_point(Hyperplane::new_with_vectors(&x,
                                                                             &y,
                                                                             &na::translate(&(z *
                                                                                 half_abc),
                                                                               &center),
                                                    ),
                                                    &center)),
                             Box::new(HalfSpace::new_with_point(Hyperplane::new_with_vectors(&x,
                                                                             &y,
                                                                             &na::translate(&-(z *
                                                                                  half_abc),
                                                                               &center),
                                                    ),
                                                    &center))];
    ComposableShape::of(shapes,
                        SetOperation::Intersection)
}
