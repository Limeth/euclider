use num::traits::NumCast;
use num::Zero;
use num::One;
use na;
use na::Point3;
use na::Vector3;
use util::CustomFloat;
use universe::entity::shape::Shape;
use universe::entity::shape::Hyperplane;
use universe::entity::shape::HalfSpace;
use universe::entity::shape::ComposableShape;
use universe::entity::shape::SetOperation;

pub type Shape3<F> = Shape<F, Point3<F>, Vector3<F>>;

pub fn cuboid<F: CustomFloat>(center: Point3<F>, abc: Vector3<F>) -> ComposableShape<F, Point3<F>, Vector3<F>> {
    let half_abc: Vector3<F> = abc / <F as NumCast>::from(2.0).unwrap();
    let x = Vector3::new(<F as One>::one(), <F as Zero>::zero(), <F as Zero>::zero());
    let y = Vector3::new(<F as Zero>::zero(), <F as One>::one(), <F as Zero>::zero());
    let z = Vector3::new(<F as Zero>::zero(), <F as Zero>::zero(), <F as One>::one());
    let shapes: Vec<Box<Shape<F, Point3<F>, Vector3<F>>>> = vec![Box::new(HalfSpace::new_with_point(Hyperplane::new_with_vectors(&y,
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
