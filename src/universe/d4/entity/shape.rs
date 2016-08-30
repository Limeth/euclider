use na::Point4;
use na::Vector4;
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

pub type Shape4<F> = Shape<F, Point4<F>, Vector4<F>>;

pub fn hypercuboid<F: CustomFloat>(center: Point4<F>, abcd: Vector4<F>) -> ComposableShape<F, Point4<F>, Vector4<F>> {
    let half_abcd: Vector4<F> = abcd / <F as NumCast>::from(2.0).unwrap();
    let x = Vector4::new(<F as One>::one(), <F as Zero>::zero(), <F as Zero>::zero(), <F as Zero>::zero());
    let y = Vector4::new(<F as Zero>::zero(), <F as One>::one(), <F as Zero>::zero(), <F as Zero>::zero());
    let z = Vector4::new(<F as Zero>::zero(), <F as Zero>::zero(), <F as One>::one(), <F as Zero>::zero());
    let w = Vector4::new(<F as Zero>::zero(), <F as Zero>::zero(), <F as Zero>::zero(), <F as One>::one());
    let shapes: Vec<Box<Shape<F, Point4<F>, Vector4<F>>>> = vec![
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
