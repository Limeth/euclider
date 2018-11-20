use ::F;
use universe::d4::Point4;
use universe::d4::Vector4;
use num::traits::NumCast;
use num::Zero;
use num::One;
use na;
use na::Norm;
use util::CustomFloat;
use universe::entity::shape::Shape;
use universe::entity::shape::Hyperplane;
use universe::entity::shape::HalfSpace;
use universe::entity::shape::ComposableShape;
use universe::entity::shape::SetOperation;

pub type Shape4 = Shape<Point4, Vector4>;

pub fn hypercuboid(center: Point4, abcd: Vector4) -> ComposableShape<Point4, Vector4> {
    let half_abcd: Vector4 = abcd / <F as NumCast>::from(2.0).unwrap();
    let x = Vector4::new(<F as One>::one(), <F as Zero>::zero(), <F as Zero>::zero(), <F as Zero>::zero());
    let y = Vector4::new(<F as Zero>::zero(), <F as One>::one(), <F as Zero>::zero(), <F as Zero>::zero());
    let z = Vector4::new(<F as Zero>::zero(), <F as Zero>::zero(), <F as One>::one(), <F as Zero>::zero());
    let w = Vector4::new(<F as Zero>::zero(), <F as Zero>::zero(), <F as Zero>::zero(), <F as One>::one());
    let shapes: Vec<Box<Shape<Point4, Vector4>>> = vec![
                            Box::new(HalfSpace::new_with_point(Hyperplane::new_with_point(x.normalize(),
                                                                             &na::translate(&(x *
                                                                                 half_abcd),
                                                                               &center),
                                                    ),
                                                    &center)),
                             Box::new(HalfSpace::new_with_point(Hyperplane::new_with_point(x.normalize(),
                                                                             &na::translate(&-(x *
                                                                                  half_abcd),
                                                                               &center),
                                                    ),
                                                    &center)),
                             Box::new(HalfSpace::new_with_point(Hyperplane::new_with_point(y.normalize(),
                                                                             &na::translate(&(y *
                                                                                 half_abcd),
                                                                               &center),
                                                    ),
                                                    &center)),
                             Box::new(HalfSpace::new_with_point(Hyperplane::new_with_point(y.normalize(),
                                                                             &na::translate(&-(y *
                                                                                  half_abcd),
                                                                               &center),
                                                    ),
                                                    &center)),
                             Box::new(HalfSpace::new_with_point(Hyperplane::new_with_point(z.normalize(),
                                                                             &na::translate(&(z *
                                                                                 half_abcd),
                                                                               &center),
                                                    ),
                                                    &center)),
                             Box::new(HalfSpace::new_with_point(Hyperplane::new_with_point(z.normalize(),
                                                                             &na::translate(&-(z *
                                                                                  half_abcd),
                                                                               &center),
                                                    ),
                                                    &center)),
                             Box::new(HalfSpace::new_with_point(Hyperplane::new_with_point(w.normalize(),
                                                                             &na::translate(&(w *
                                                                                 half_abcd),
                                                                               &center),
                                                    ),
                                                    &center)),
                             Box::new(HalfSpace::new_with_point(Hyperplane::new_with_point(w.normalize(),
                                                                             &na::translate(&-(w *
                                                                                  half_abcd),
                                                                               &center),
                                                    ),
                                                    &center))
                             ];
    ComposableShape::of(shapes,
                        SetOperation::Intersection)
}
