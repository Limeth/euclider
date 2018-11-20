use std::marker::PhantomData;
use util::VecLazy;
use util::IterLazy;
use std::fmt;
use std::fmt::Debug;
use std::fmt::Display;
use std::time::Duration;
use std::any::TypeId;
use std::any::Any;
use std::sync::Arc;
use std::iter;
use palette::Rgba;
use universe::entity::Traceable;
use universe::entity::material::Material;
use util::CustomFloat;
use util::CustomPoint;
use util::CustomVector;
use util::HasId;
use util::Provider;
use util::TypePairMap;
use util::PossiblyImmediateIterator;
use num::Zero;
use num::One;
use num::NumCast;
use mopa;
use na;
use na::Cross;
use smallvec::SmallVec;
use smallvec::IntoIter;

/// Ties a `Material` the ray is passing through and a `Shape` the ray is intersecting to a
/// `GeneralIntersector`
pub type GeneralIntersectors<F, P, V> = TypePairMap<Box<GeneralIntersector<F, P, V>>>;

/// Computes the intersections of a ray in a given `Material` with a given `Shape`.
/// The ray is originating in the given `Point` with a direction of the given `Vector`.
// Send + Sync must be at the end of the type alias definition.
pub type GeneralIntersector<F, P, V> = (Fn(&P,
                                           &V,
                                           &Material<F, P, V>,
                                           &Shape<F, P, V>,
                                           Intersector<F, P, V>
                                        ) -> GeneralIntersectionMarcher<F, P, V>) + Send + Sync;

pub type ImmediateIntersections<F, P, V> = [Intersection<F, P, V>; 8];

pub type GeneralIntersectionMarcher<F: CustomFloat, P: CustomPoint<F, V>, V: CustomVector<F, P>> = PossiblyImmediateIterator<Intersection<F, P, V>, ImmediateIntersections<F, P, V>>;

pub type IntersectionMarcher<F, P, V> = Iterator<Item = Intersection<F, P, V>>;

/// Computes the intersections of a ray in a given `Material` with a given `Shape`
/// with a predefined `Point` of origin and directional `Vector`.
// TODO: It feels wrong to have a type alias to a reference of another type
pub type Intersector<'a, F, P, V> = &'a Fn(&Material<F, P, V>, &Shape<F, P, V>)
                                           -> IntersectionProvider<F, P, V>;

/// Calls the `trace` method on the current Universe and returns the resulting color.
// TODO: It feels wrong to have a type alias to a reference of another type
pub type ColorTracer<'a, F, P, V> = &'a Fn(&Duration, &Traceable<F, P, V>, &P, &V) -> Rgba<F>;

/// Calls the `trace_path` method on the current Universe and returns the resulting location and
/// vector.
// TODO: It feels wrong to have a type alias to a reference of another type
pub type PathTracer<'a, F, P, V> = &'a Fn(&Duration, &F, &Traceable<F, P, V>, &P, &V) -> (P, V);

/// Calls the `trace_path` method on the current Universe and returns the resulting location and
/// vector.
// TODO: It feels wrong to have a type alias to a reference of another type
pub type MaterialFinder<'a, F, P, V> = &'a Fn(&P) -> Option<&'a Traceable<F, P, V>>;

pub trait Shape<F: CustomFloat, P: CustomPoint<F, V>, V: CustomVector<F, P>>
    where Self: HasId + Debug + Display + mopa::Any + Send + Sync
{
    fn is_point_inside(&self, point: &P) -> bool;
}

mopafy!(Shape<F: CustomFloat, P: CustomPoint<F, V>, V: CustomVector<F, P>>);

#[macro_export]
macro_rules! shape {
    ($($t:tt)*) => {
        has_id!($($t)*);
        name_as_display!($($t)*);
    }
}

#[derive(Debug, Copy, Clone)]
pub struct Intersection<F: CustomFloat, P: CustomPoint<F, V>, V: CustomVector<F, P>> {
    pub location: P,
    pub direction: V,
    pub normal: V,
    pub distance: F,
    pub float_precision: PhantomData<F>,
    pub vector_dimensions: PhantomData<V>,
}

impl<F: CustomFloat, P: CustomPoint<F, V>, V: CustomVector<F, P>> Intersection<F, P, V> {
    pub fn new(location: P, direction: V, normal: V, distance: F) -> Intersection<F, P, V> {
        Intersection {
            location: location,
            direction: direction,
            normal: normal,
            distance: distance,
            float_precision: PhantomData,
            vector_dimensions: PhantomData,
        }
    }
}

#[derive(Copy, Clone)]
pub struct TracingContext<'a,
                          F: 'a + CustomFloat,
                          P: 'a + CustomPoint<F, V>,
                          V: 'a + CustomVector<F, P>>
{
    pub debugging: bool,
    pub time: Duration,
    pub origin_traceable: &'a Traceable<F, P, V>,
    pub origin_location: P,
    pub origin_direction: V,
    pub intersection_traceable: &'a Traceable<F, P, V>,
    pub intersection: Intersection<F, P, V>,
    pub intersection_normal_closer: V,
    pub exiting: bool,
}

#[derive(Copy, Clone)]
pub struct ColorTracingContext<'a,
                               F: 'a + CustomFloat,
                               P: 'a + CustomPoint<F, V>,
                               V: 'a + CustomVector<F, P>>
{
    pub general: TracingContext<'a, F, P, V>,
    pub depth_remaining: &'a u32,
    pub trace: ColorTracer<'a, F, P, V>,
    pub material_at: MaterialFinder<'a, F, P, V>,
}

#[derive(Copy, Clone)]
pub struct PathTracingContext<'a,
                              F: 'a + CustomFloat,
                              P: 'a + CustomPoint<F, V>,
                              V: 'a + CustomVector<F, P>>
{
    pub general: TracingContext<'a, F, P, V>,
    pub distance: &'a F,
    pub trace: PathTracer<'a, F, P, V>,
    pub material_at: MaterialFinder<'a, F, P, V>,
}

#[allow(dead_code)]
#[derive(Debug, Copy, Clone)]
pub enum SetOperation {
    Union, // A + B
    Intersection, // A && B
    Complement, // A - B
    SymmetricDifference, // A ^ B
}

debug_as_display!(SetOperation);

pub type IntersectionProvider<F, P, V> = Provider<Intersection<F, P, V>, ImmediateIntersections<F, P, V>>;

struct ComposableShapeIterator<F: CustomFloat, P: CustomPoint<F, V>, V: CustomVector<F, P>> {
    shape_a: Arc<Box<Shape<F, P, V>>>,
    shape_b: Arc<Box<Shape<F, P, V>>>,
    provider_a: IntersectionProvider<F, P, V>,
    provider_b: IntersectionProvider<F, P, V>,
    index_a: usize,
    index_b: usize,
}

impl<F: CustomFloat, P: CustomPoint<F, V>, V: CustomVector<F, P>> ComposableShapeIterator<F, P, V> {
    fn new(shape_a: Arc<Box<Shape<F, P, V>>>,
           shape_b: Arc<Box<Shape<F, P, V>>>,
           provider_a: IntersectionProvider<F, P, V>,
           provider_b: IntersectionProvider<F, P, V>)
           -> ComposableShapeIterator<F, P, V> {
        ComposableShapeIterator {
            shape_a: shape_a,
            shape_b: shape_b,
            provider_a: provider_a,
            provider_b: provider_b,
            index_a: 0,
            index_b: 0,
        }
    }
}

struct UnionIterator<F: CustomFloat, P: CustomPoint<F, V>, V: CustomVector<F, P>> {
    data: ComposableShapeIterator<F, P, V>,
}

impl<F: CustomFloat, P: CustomPoint<F, V>, V: CustomVector<F, P>> UnionIterator<F, P, V> {
    fn new(shape_a: Arc<Box<Shape<F, P, V>>>,
           shape_b: Arc<Box<Shape<F, P, V>>>,
           provider_a: IntersectionProvider<F, P, V>,
           provider_b: IntersectionProvider<F, P, V>)
           -> UnionIterator<F, P, V> {
        UnionIterator {
            data: ComposableShapeIterator::new(shape_a, shape_b, provider_a, provider_b),
        }
    }
}

impl<F: CustomFloat, P: CustomPoint<F, V>, V: CustomVector<F, P>> Iterator for UnionIterator<F,
                                                                                             P,
                                                                                             V> {
    type Item = Intersection<F, P, V>;


    // Should return the following intersections:
    // --> [a] [b] [a[a+b]b]
    //     ^-^ ^-^ ^-------^
    #[allow(useless_let_if_seq)]
    fn next(&mut self) -> Option<Self::Item> {
        loop {
            let intersection_a = self.data.provider_a.get(self.data.index_a);
            let intersection_b = self.data.provider_b.get(self.data.index_b);

            if intersection_a.is_some() {
                let unwrapped_a = intersection_a.unwrap();

                if intersection_b.is_some() {
                    let unwrapped_b = intersection_b.unwrap();
                    let closer: Intersection<F, P, V>;
                    let closer_index: &mut usize;
                    let further_shape: &Shape<F, P, V>;

                    if unwrapped_a.distance < unwrapped_b.distance {
                        closer = unwrapped_a;
                        closer_index = &mut self.data.index_a;
                        further_shape = self.data.shape_b.as_ref().as_ref();
                    } else {
                        closer = unwrapped_b;
                        closer_index = &mut self.data.index_b;
                        further_shape = self.data.shape_a.as_ref().as_ref();
                    }

                    if !further_shape.is_point_inside(&closer.location) {
                        *closer_index += 1;
                        return Some(closer);
                    }

                    *closer_index += 1;
                } else {
                    self.data.index_a += 1;

                    if self.data.shape_b.is_point_inside(&unwrapped_a.location) {
                        return None;
                    }

                    return intersection_a;
                }
            } else {
                if intersection_b.is_some() {
                    let unwrapped_b = intersection_b.unwrap();
                    self.data.index_b += 1;

                    if self.data.shape_a.is_point_inside(&unwrapped_b.location) {
                        return None;
                    }
                }

                return intersection_b;
            }
        }
    }
}

struct IntersectionIterator<F: CustomFloat, P: CustomPoint<F, V>, V: CustomVector<F, P>> {
    data: ComposableShapeIterator<F, P, V>,
}

impl<F: CustomFloat, P: CustomPoint<F, V>, V: CustomVector<F, P>> IntersectionIterator<F, P, V> {
    fn new(shape_a: Arc<Box<Shape<F, P, V>>>,
           shape_b: Arc<Box<Shape<F, P, V>>>,
           provider_a: IntersectionProvider<F, P, V>,
           provider_b: IntersectionProvider<F, P, V>)
           -> IntersectionIterator<F, P, V> {
        IntersectionIterator {
            data: ComposableShapeIterator::new(shape_a, shape_b, provider_a, provider_b),
        }
    }
}

impl<F: CustomFloat, P: CustomPoint<F, V>, V: CustomVector<F, P>>
        Iterator for IntersectionIterator<F, P, V> {
    type Item = Intersection<F, P, V>;

// Should return the following intersections:
// --> [a] [b] [a[a+b]b]
//               ^---^
    #[allow(useless_let_if_seq)]
    fn next(&mut self) -> Option<Self::Item> {
        loop {
            let intersection_a = self.data.provider_a.get(self.data.index_a);
            let intersection_b = self.data.provider_b.get(self.data.index_b);

            if intersection_a.is_some() {
                if intersection_b.is_some() {
                    let unwrapped_a = intersection_a.unwrap();
                    let unwrapped_b = intersection_b.unwrap();
                    let closer: Intersection<F, P, V>;
                    let closer_index: &mut usize;
                    let further_shape: &Shape<F, P, V>;

                    if unwrapped_a.distance < unwrapped_b.distance {
                        closer = unwrapped_a;
                        closer_index = &mut self.data.index_a;
                        further_shape = self.data.shape_b.as_ref().as_ref();
                    } else {
                        closer = unwrapped_b;
                        closer_index = &mut self.data.index_b;
                        further_shape = self.data.shape_a.as_ref().as_ref();
                    }

                    *closer_index += 1;

                    if further_shape.is_point_inside(&closer.location) {
                        return Some(closer);
                    }
                } else {
                    self.data.index_a += 1;

                    if self.data.shape_b.is_point_inside(&intersection_a.unwrap().location) {
                        return intersection_a;
                    }

                    return None;
                }
            } else {
                if intersection_b.is_some() {
                    self.data.index_b += 1;

                    if self.data.shape_a.is_point_inside(&intersection_b.unwrap().location) {
                        return intersection_b;
                    }
                }

                return None;
            }
        }
    }
}

struct ComplementIterator<F: CustomFloat, P: CustomPoint<F, V>, V: CustomVector<F, P>> {
    data: ComposableShapeIterator<F, P, V>,
}

impl<F: CustomFloat, P: CustomPoint<F, V>, V: CustomVector<F, P>> ComplementIterator<F, P, V> {
    fn new(shape_a: Arc<Box<Shape<F, P, V>>>,
           shape_b: Arc<Box<Shape<F, P, V>>>,
           provider_a: IntersectionProvider<F, P, V>,
           provider_b: IntersectionProvider<F, P, V>)
           -> ComplementIterator<F, P, V> {
        ComplementIterator {
            data: ComposableShapeIterator::new(shape_a, shape_b, provider_a, provider_b),
        }
    }
}

impl<F: CustomFloat, P: CustomPoint<F, V>, V: CustomVector<F, P>> Iterator for ComplementIterator<F,
                                                                                                  P,
                                                                                                  V> {
    type Item = Intersection<F, P, V>;

    // Should return the following intersections:
    // --> [a] [b] [a[a+b]b] [b[a+b]a]
    //     ^-^     ^-^             ^ ^
    fn next(&mut self) -> Option<Self::Item> {
        loop {
            let intersection_a = self.data.provider_a.get(self.data.index_a);
            let intersection_b = self.data.provider_b.get(self.data.index_b);

            if intersection_a.is_some() {
                if intersection_b.is_some() {
                    let unwrapped_a = intersection_a.unwrap();
                    let unwrapped_b = intersection_b.unwrap();

                    if unwrapped_a.distance < unwrapped_b.distance {
                        self.data.index_a += 1;

                        if !self.data.shape_b.is_point_inside(&unwrapped_a.location) {
                            return Some(unwrapped_a);
                        }
                    } else {
                        self.data.index_b += 1;

                        if self.data.shape_a.is_point_inside(&unwrapped_b.location) {
                            let mut inverted_b = unwrapped_b;
                            inverted_b.normal = -inverted_b.normal;
                            return Some(inverted_b);
                        }
                    }
                } else {
                    return intersection_a;
                }
            } else {
                if intersection_b.is_some() {
                    let unwrapped_b = intersection_b.unwrap();

                    self.data.index_b += 1;

                    if self.data.shape_a.is_point_inside(&unwrapped_b.location) {
                        let mut inverted_b = unwrapped_b;
                        inverted_b.normal = -inverted_b.normal;
                        return Some(inverted_b);
                    }
                }

                return None;
            }
        }
    }
}

struct SymmetricDifferenceIterator<F: CustomFloat, P: CustomPoint<F, V>, V: CustomVector<F, P>> {
    data: ComposableShapeIterator<F, P, V>,
}

impl<F: CustomFloat, P: CustomPoint<F, V>, V: CustomVector<F, P>> SymmetricDifferenceIterator<F,
                                                                                              P,
                                                                                              V> {
    fn new(shape_a: Arc<Box<Shape<F, P, V>>>,
           shape_b: Arc<Box<Shape<F, P, V>>>,
           provider_a: IntersectionProvider<F, P, V>,
           provider_b: IntersectionProvider<F, P, V>)
           -> SymmetricDifferenceIterator<F, P, V> {
        SymmetricDifferenceIterator {
            data: ComposableShapeIterator::new(shape_a, shape_b, provider_a, provider_b),
        }
    }
}

impl<F: CustomFloat, P: CustomPoint<F, V>, V: CustomVector<F, P>>
        Iterator for SymmetricDifferenceIterator<F, P, V> {
    type Item = Intersection<F, P, V>;

// Should return the following intersections:
// --> [a] [b] [a[a+b]b]
//     ^-^ ^-^ ^-^   ^-^
    #[allow(useless_let_if_seq)]
    fn next(&mut self) -> Option<Self::Item> {
        let intersection_a = self.data.provider_a.get(self.data.index_a);
        let intersection_b = self.data.provider_b.get(self.data.index_b);

        if intersection_a.is_some() {
            if intersection_b.is_some() {
                let unwrapped_a = intersection_a.unwrap();
                let unwrapped_b = intersection_b.unwrap();
                let closer: Intersection<F, P, V>;
                let closer_index: &mut usize;
                let further_shape: &Shape<F, P, V>;

                if unwrapped_a.distance < unwrapped_b.distance {
                    closer = unwrapped_a;
                    closer_index = &mut self.data.index_a;
                    further_shape = self.data.shape_b.as_ref().as_ref();
                } else {
                    closer = unwrapped_b;
                    closer_index = &mut self.data.index_b;
                    further_shape = self.data.shape_a.as_ref().as_ref();
                }

                if further_shape.is_point_inside(&closer.location) {
                    *closer_index += 1;
                    let mut closer_inverted = closer;
                    closer_inverted.normal = -closer_inverted.normal;
                    return Some(closer_inverted);
                }

                *closer_index += 1;

                Some(closer)
            } else {
                let unwrapped_a = intersection_a.unwrap();

                self.data.index_a += 1;

                if self.data.shape_b.is_point_inside(&unwrapped_a.location) {
                    let mut closer_inverted = unwrapped_a;
                    closer_inverted.normal = -closer_inverted.normal;
                    return Some(closer_inverted);
                }

                intersection_a
            }
        } else {
            if intersection_b.is_some() {
                let unwrapped_b = intersection_b.unwrap();

                self.data.index_b += 1;

                if self.data.shape_a.is_point_inside(&unwrapped_b.location) {
                    let mut closer_inverted = unwrapped_b;
                    closer_inverted.normal = -closer_inverted.normal;
                    return Some(closer_inverted);
                }
            }

            intersection_b
        }
    }
}

#[derive(Debug)]
pub struct ComposableShape<F: CustomFloat, P: CustomPoint<F, V>, V: CustomVector<F, P>> {
    pub a: Arc<Box<Shape<F, P, V>>>,
    pub b: Arc<Box<Shape<F, P, V>>>,
    pub operation: SetOperation,
    marker: PhantomData<Shape<F, P, V>>,
}

shape!(ComposableShape<F: 'static + CustomFloat, P: 'static + CustomPoint<F, V>, V: 'static + CustomVector<F, P>>);

impl<F: CustomFloat, P: CustomPoint<F, V>, V: CustomVector<F, P>> ComposableShape<F, P, V> {
    pub fn new<A: Shape<F, P, V> + 'static, B: Shape<F, P, V> + 'static>
        (a: A,
         b: B,
         operation: SetOperation)
         -> ComposableShape<F, P, V> {
        ComposableShape {
            a: Arc::new(Box::new(a)),
            b: Arc::new(Box::new(b)),
            operation: operation,
            marker: PhantomData,
        }
    }

    pub fn of<I: IntoIterator<Item = Box<Shape<F, P, V>>>>(shapes: I,
                                                          operation: SetOperation)
                                                          -> ComposableShape<F, P, V> {
        const PANIC: &str = "2 or more `Shape`s are needed to construct a `ComposableShape`.";
        let mut shapes = shapes.into_iter();
        let mut result = ComposableShape {
            a: Arc::new(shapes.next().expect(PANIC)),
            b: Arc::new(shapes.next().expect(PANIC)),
            operation: operation,
            marker: PhantomData,
        };

        for shape in shapes {
            result = ComposableShape {
                a: Arc::new(Box::new(result)),
                b: Arc::new(shape),
                operation: operation,
                marker: PhantomData,
            };
        }

        result
    }

    #[allow(unused_variables)]
    pub fn intersect_linear(location: &P,
                               direction: &V,
                               vacuum: &Material<F, P, V>,
                               shape: &Shape<F, P, V>,
                               intersect: Intersector<F, P, V>)
                               -> GeneralIntersectionMarcher<F, P, V> {
        let composed: &ComposableShape<F, P, V> =
            shape.as_any().downcast_ref::<ComposableShape<F, P, V>>().unwrap();
        let provider_a = intersect(vacuum, composed.a.as_ref().as_ref());
        let provider_b = intersect(vacuum, composed.b.as_ref().as_ref());
        PossiblyImmediateIterator::Dynamic(match composed.operation {
            SetOperation::Union => {
                Box::new(UnionIterator::new(Arc::clone(&composed.a),
                                            Arc::clone(&composed.b),
                                            provider_a,
                                            provider_b))
            }
            SetOperation::Intersection => {
                Box::new(IntersectionIterator::new(Arc::clone(&composed.a),
                                                   Arc::clone(&composed.b),
                                                   provider_a,
                                                   provider_b))
            }
            SetOperation::Complement => {
                Box::new(ComplementIterator::new(Arc::clone(&composed.a),
                                                 Arc::clone(&composed.b),
                                                 provider_a,
                                                 provider_b))
            }
            SetOperation::SymmetricDifference => {
                Box::new(SymmetricDifferenceIterator::new(Arc::clone(&composed.a),
                                                          Arc::clone(&composed.b),
                                                          provider_a,
                                                          provider_b))
            }
        })
    }
}

impl<F: 'static + CustomFloat, P: 'static + CustomPoint<F, V>, V: 'static + CustomVector<F, P>> Shape<F, P, V>
        for ComposableShape<F, P, V> {
    fn is_point_inside(&self, point: &P) -> bool {
        match self.operation {
            SetOperation::Union =>
                self.a.is_point_inside(point) || self.b.is_point_inside(point),
            SetOperation::Intersection =>
                self.a.is_point_inside(point) && self.b.is_point_inside(point),
            SetOperation::Complement =>
                self.a.is_point_inside(point) && !self.b.is_point_inside(point),
            SetOperation::SymmetricDifference =>
                self.a.is_point_inside(point) ^ self.b.is_point_inside(point),
        }
    }
}

#[derive(Default, Debug)]
pub struct VoidShape {}

shape!(VoidShape);

impl VoidShape {
    pub fn new() -> Self {
        VoidShape {}
    }
}

impl<F: CustomFloat, P: CustomPoint<F, V>, V: CustomVector<F, P>> Shape<F, P, V> for VoidShape {
    #[allow(unused_variables)]
    fn is_point_inside(&self, point: &P) -> bool {
        true
    }
}

#[allow(unused_variables)]
pub fn intersect_void<F: CustomFloat, P: CustomPoint<F, V>, V: CustomVector<F, P>>
    (location: &P,
     direction: &V,
     material: &Material<F, P, V>,
     void: &Shape<F, P, V>,
     intersect: Intersector<F, P, V>)
     -> GeneralIntersectionMarcher<F, P, V> {
    void.as_any().downcast_ref::<VoidShape>().unwrap();
    PossiblyImmediateIterator::Immediate(SmallVec::new().into_iter())
}

#[derive(Debug)]
pub struct Sphere<F: CustomFloat, P: CustomPoint<F, V>, V: CustomVector<F, P>> {
    pub location: P,
    pub radius: F,
    marker_vector: PhantomData<V>,
}

shape!(Sphere<F: CustomFloat, P: CustomPoint<F, V>, V: CustomVector<F, P>>);

impl<F: CustomFloat, P: CustomPoint<F, V>, V: CustomVector<F, P>> Sphere<F, P, V> {
    pub fn new(location: P, radius: F) -> Sphere<F, P, V> {
        Sphere {
            location: location,
            radius: radius,
            marker_vector: PhantomData,
        }
    }

    #[allow(unused_variables)]
    pub fn intersect_linear
        (location: &P,
         direction: &V,
         vacuum: &Material<F, P, V>,
         sphere: &Shape<F, P, V>,
         intersect: Intersector<F, P, V>)
         -> GeneralIntersectionMarcher<F, P, V> {
        let sphere: &Sphere<F, P, V> =
            sphere.as_any().downcast_ref::<Sphere<F, P, V>>().unwrap();

        let rel = *location - sphere.location;
        let a: F = direction.norm_squared();
        let b: F = <F as NumCast>::from(2.0).unwrap() * direction.dot(&rel);
        let c: F = rel.norm_squared() - sphere.radius * sphere.radius;

        // Discriminant = b^2 - 4*a*c
        let d: F = b * b - <F as NumCast>::from(4.0).unwrap() * a * c;

        if d < <F as Zero>::zero() {
            return PossiblyImmediateIterator::Immediate(SmallVec::new().into_iter());
        }

        let d_sqrt = d.sqrt();
        let mut t_first: Option<F> = None;  // The smallest non-negative vector modifier
        let mut t_second: Option<F> = None;  // The second smallest non-negative vector modifier
        let t1: F = (-b - d_sqrt) / (<F as NumCast>::from(2.0).unwrap() * a);
        let t2: F = (-b + d_sqrt) / (<F as NumCast>::from(2.0).unwrap() * a);

        if t1 >= <F as Zero>::zero() {
            t_first = Some(t1);

            if t2 >= <F as Zero>::zero() {
                t_second = Some(t2);
            }
        } else if t2 >= <F as Zero>::zero() {
            t_first = Some(t2);
        }

        if t_first.is_none() {
            // Don't trace in the opposite direction
            return PossiblyImmediateIterator::Immediate(SmallVec::new().into_iter());
        }

        let t_first = t_first.unwrap();
        let mut closures: VecLazy<Intersection<F, P, V>> = Vec::new();
        // Move the following variables inside the closures.
        // This lets the closures move outside the scope.
        let direction = *direction;
        let location = *location;
        let sphere_location = sphere.location;
        let mut intersections = SmallVec::with_capacity(2);

        {
            let result_vector = direction * t_first;
            let result_point = location + result_vector;

            let mut normal = result_point - sphere_location;
            normal = na::normalize(&normal);

            intersections.push(Intersection::new(result_point,
                                                 direction,
                                                 normal,
                                                 t_first));
        }

        if let Some(t_second) = t_second {
            let result_vector = direction * t_second;
            let result_point = location + result_vector;

            let mut normal = result_point - sphere_location;
            normal = na::normalize(&normal);

            intersections.push(Intersection::new(result_point,
                                                 direction,
                                                 normal,
                                                 t_second));
        }

        PossiblyImmediateIterator::Immediate(intersections.into_iter())
    }
}

impl<F: CustomFloat, P: CustomPoint<F, V>, V: CustomVector<F, P>> Shape<F, P, V> for Sphere<F, P, V> {
    fn is_point_inside(&self, point: &P) -> bool {
        na::distance_squared(&self.location, point) <= self.radius * self.radius
    }
}

#[derive(Debug)]
pub struct Hyperplane<F: CustomFloat, P: CustomPoint<F, V>, V: CustomVector<F, P>> {
    pub normal: V,
    pub constant: F,
    marker: PhantomData<P>,
}

shape!(Hyperplane<F: CustomFloat, P: CustomPoint<F, V>, V: CustomVector<F, P>>);

impl<F: CustomFloat, P: CustomPoint<F, V>, V: CustomVector<F, P>> Hyperplane<F, P, V> {
    pub fn new(normal: V, constant: F) -> Self {
        assert!(normal.norm_squared() > <F as Zero>::zero(),
                "Cannot have a normal with length of 0.");

        Hyperplane {
            normal: normal,
            constant: constant,
            marker: PhantomData,
        }
    }

    pub fn new_with_point(normal: V, point: &P) -> Self {
        // D = -(A*x + B*y + C*z)
        let constant = -na::dot(&normal, point.as_vector());

        Self::new(normal, constant)
    }

    pub fn new_with_vectors(vector_a: &V,
                            vector_b: &V,
                            point: &P)
                            -> Self where V: Cross<CrossProductType=V> {
        // A*x + B*y + C*z + D = 0
        let normal = na::cross(vector_a, vector_b);

        Self::new_with_point(normal, point)
    }

    #[allow(unused_variables)]
    pub fn intersect_linear(location: &P,
                               direction: &V,
                               vacuum: &Material<F, P, V>,
                               shape: &Shape<F, P, V>,
                               intersect: Intersector<F, P, V>)
                               -> GeneralIntersectionMarcher<F, P, V> {
        let plane: &Hyperplane<F, P, V> = shape.as_any().downcast_ref::<Hyperplane<F, P, V>>().unwrap();

        // A*x + B*y + C*z + D = 0

        let t: F = -(na::dot(&plane.normal, location.as_vector()) + plane.constant) /
                   na::dot(&plane.normal, direction);

        if t < <F as Zero>::zero() {
            return PossiblyImmediateIterator::Immediate(SmallVec::new().into_iter());
        }

        let result_vector = *direction * t;
        let result_point = result_vector.translate(location);

        let normal = plane.normal;

        let mut intersections = SmallVec::with_capacity(1);

        intersections.push(Intersection::new(result_point,
                                             *direction,
                                             normal,
                                             t));

        PossiblyImmediateIterator::Immediate(intersections.into_iter())
    }
}

impl<F: CustomFloat, P: CustomPoint<F, V>, V: CustomVector<F, P>> Shape<F, P, V> for Hyperplane<F, P, V> {
    #[allow(unused_variables)]
    fn is_point_inside(&self, point: &P) -> bool {
        false
    }
}

#[derive(Debug)]
pub struct HalfSpace<F: CustomFloat, P: CustomPoint<F, V>, V: CustomVector<F, P>> {
    pub plane: Hyperplane<F, P, V>,
    pub signum: F,
}

shape!(HalfSpace<F: CustomFloat, P: CustomPoint<F, V>, V: CustomVector<F, P>>);

impl<F: CustomFloat, P: CustomPoint<F, V>, V: CustomVector<F, P>> HalfSpace<F, P, V> {
    pub fn new(plane: Hyperplane<F, P, V>, mut signum: F) -> HalfSpace<F, P, V> {
        signum /= signum.abs();

        HalfSpace {
            plane: plane,
            signum: signum,
        }
    }

    pub fn new_with_point(plane: Hyperplane<F, P, V>, point_inside: &P) -> HalfSpace<F, P, V> {
        let identifier: F = na::dot(&plane.normal, point_inside.as_vector()) + plane.constant;

        Self::new(plane, identifier)
    }

    pub fn intersect_linear(location: &P,
                               direction: &V,
                               vacuum: &Material<F, P, V>,
                               shape: &Shape<F, P, V>,
                               intersect: Intersector<F, P, V>)
                               -> GeneralIntersectionMarcher<F, P, V> {
        let halfspace: &HalfSpace<F, P,V> = shape.as_any().downcast_ref::<HalfSpace<F, P, V>>().unwrap();
        let intersection = Hyperplane::<F, P, V>::intersect_linear(location,
                                                            direction,
                                                            vacuum,
                                                            &halfspace.plane,
                                                            intersect)
            .next();

        // Works so far, not sure why
        if intersection.is_some() {
            let mut intersection = intersection.unwrap();
            intersection.normal *= -halfspace.signum;

            let mut intersections = SmallVec::with_capacity(1);

            intersections.push(intersection);

            return PossiblyImmediateIterator::Immediate(intersections.into_iter());
        }

        PossiblyImmediateIterator::Immediate(SmallVec::new().into_iter())
    }
}

impl<F: CustomFloat, P: CustomPoint<F, V>, V: CustomVector<F, P>> Shape<F, P, V> for HalfSpace<F, P, V> {
    fn is_point_inside(&self, point: &P) -> bool {
        // A*x + B*y + C*z + D = 0
        // ~~~~~~~~~~~~~~~ dot
        let result: F = na::dot(&self.plane.normal, point.as_vector()) + self.plane.constant;

        self.signum == result.signum()
    }
}

#[derive(Debug)]
pub struct Cylinder<F: CustomFloat, P: CustomPoint<F, V>, V: CustomVector<F, P>> {
    pub center: P,  // Must be normalized; TODO: update after upgrading nalgebra
    pub direction: V,
    pub radius: F,
}

shape!(Cylinder<F: CustomFloat, P: CustomPoint<F, V>, V: CustomVector<F, P>>);

impl<F: CustomFloat, P: CustomPoint<F, V>, V: CustomVector<F, P>> Cylinder<F, P, V> {
    pub fn new(center: P, direction: &V, radius: F) -> Self {
        assert!(direction.norm_squared() > <F as Zero>::zero(),
                "Cannot have a direction with length of 0.");
        assert!(radius > <F as Zero>::zero(),
                "The radius must be positive.");

        Cylinder {
            center: center,
            direction: direction.normalize(),
            radius: radius,
        }
    }

    pub fn new_with_height(center: P, direction: &V, radius: F, height: F) -> ComposableShape<F, P, V> {
        let normalized_direction = direction.normalize();
        let half_height = height / (<F as One>::one() + <F as One>::one());
        let shapes: Vec<Box<Shape<F, P, V>>> = vec![
                Box::new(Self::new(center, direction, radius)),
                Box::new(HalfSpace::new_with_point(
                    Hyperplane::new_with_point(normalized_direction,
                        &(normalized_direction * half_height).translate(&center)),
                    &center
                )),
                Box::new(HalfSpace::new_with_point(
                    Hyperplane::new_with_point(normalized_direction,
                        &(normalized_direction * -half_height).translate(&center)),
                    &center
                ))
            ];

        ComposableShape::of(
            shapes,
            SetOperation::Intersection
        )
    }

    fn get_closest_point_on_axis(&self, to: &P) -> P {
        (self.direction * self.direction.dot(&(*to - self.center)))
            .translate(&self.center)
    }

    #[allow(unused_variables)]
    pub fn intersect_linear(location: &P,
                               direction: &V,
                               vacuum: &Material<F, P, V>,
                               shape: &Shape<F, P, V>,
                               intersect: Intersector<F, P, V>)
                               -> GeneralIntersectionMarcher<F, P, V> {
        let cylinder: &Cylinder<F, P, V> = shape.as_any().downcast_ref::<Cylinder<F, P, V>>().unwrap();

        // The intersection of a line and an infinite cylinder is calculated as follows:
        // `|(Q - {S_c + [v_c dot (Q - S_c)] * v_c})|^2 - r^2 = 0`
        // where `Q` is a point on the line (typically `S_l + t * v_l`),
        //       `S_c` is a point on the axis of the cylinder,
        //       `v_c` is the direction of the cylinder axis (must be normalized),
        //       `r` is the radius of the cylinder.
        //
        // `[v_c dot (Q - S_c)] * v_c` is the vector from `S_c` to the closest point
        // to the ray on the cylinder axis.
        //
        // After substituting `Q = S_l + t * v_l`, we can arrange as `A * t^2 + B * t + C = 0`.

        let a_vec = *direction - cylinder.direction * direction.dot(&cylinder.direction);
        let delta_location = *location - cylinder.center;
        let c_vec = delta_location - cylinder.direction * delta_location.dot(&cylinder.direction);
        let a = a_vec.norm_squared();
        let b = (<F as One>::one() + <F as One>::one()) * a_vec.dot(&c_vec);
        let c = c_vec.norm_squared() - cylinder.radius * cylinder.radius;

        // Discriminant = b^2 - 4*a*c
        let d: F = b * b - <F as NumCast>::from(4.0).unwrap() * a * c;

        if d < <F as Zero>::zero() {
            return PossiblyImmediateIterator::Immediate(SmallVec::new().into_iter());
        }

        let d_sqrt = d.sqrt();
        let mut t_first: Option<F> = None;  // The smallest non-negative vector modifier
        let mut t_second: Option<F> = None;  // The second smallest non-negative vector modifier
        let t1: F = (-b - d_sqrt) / (<F as NumCast>::from(2.0).unwrap() * a);
        let t2: F = (-b + d_sqrt) / (<F as NumCast>::from(2.0).unwrap() * a);

        if t1 >= <F as Zero>::zero() {
            t_first = Some(t1);

            if t2 >= <F as Zero>::zero() {
                t_second = Some(t2);
            }
        } else if t2 >= <F as Zero>::zero() {
            t_first = Some(t2);
        }

        if t_first.is_none() {
            // Don't trace in the opposite direction
            return PossiblyImmediateIterator::Immediate(SmallVec::new().into_iter());
        }

        let t_first = t_first.unwrap();
        let mut closures: VecLazy<Intersection<F, P, V>> = Vec::new();
        // Move the following variables inside the closures.
        // This lets the closures move outside the scope.
        let direction = *direction;
        let location = *location;
        // `[v_c dot (Q - S_c)] * v_c` is the vector from `S_c` to the closest point
        let result_vector = direction * t_first;
        let result_point_1 = location + result_vector;
        let closest_point_on_axis = cylinder.get_closest_point_on_axis(&result_point_1);

        let mut intersections = SmallVec::with_capacity(2);

        {
            let mut normal = result_point_1 - closest_point_on_axis;
            normal = na::normalize(&normal);

            intersections.push(Intersection::new(result_point_1,
                                                 direction,
                                                 normal,
                                                 t_first));
        }

        if let Some(t_second) = t_second {
            let result_vector = direction * t_second;
            let result_point_2 = location + result_vector;

            let mut normal = result_point_2 - closest_point_on_axis;
            normal = na::normalize(&normal);

            intersections.push(Intersection::new(result_point_2,
                                                 direction,
                                                 normal,
                                                 t_second));
        }

        PossiblyImmediateIterator::Immediate(intersections.into_iter())
    }
}

impl<F: CustomFloat, P: CustomPoint<F, V>, V: CustomVector<F, P>> Shape<F, P, V> for Cylinder<F, P, V> {
    #[allow(unused_variables)]
    fn is_point_inside(&self, point: &P) -> bool {
        let closest_point_on_axis = self.get_closest_point_on_axis(point);
        let vector_to_point = *point - closest_point_on_axis;

        vector_to_point.norm_squared() <= self.radius * self.radius
    }
}

#[cfg(test)]
mod tests {
    use universe::entity::material::Vacuum;
    use na::Point2;
    use na::Vector2;
    use na::ApproxEq;
    use super::*;

    #[test]
    fn intersect_sphere_linear() {
        let mut marcher = Sphere::intersect_linear(
            &Point2::new(0.0, 0.0),
            &Vector2::new(1.0, 0.0),
            &Vacuum::new(),
            &Sphere::new(
                Point2::new(2.0, 0.0),
                1.0
            ),
            &|_, _| { unimplemented!() }
        );

        let first = marcher.next().unwrap();
        let second = marcher.next().unwrap();

        assert_eq!(first.location, Point2::new(1.0, 0.0));
        assert_eq!(first.direction, Vector2::new(1.0, 0.0));
        assert_eq!(first.normal, Vector2::new(-1.0, 0.0));
        assert!(first.distance.approx_eq_ulps(&1.0, 2));
        assert_eq!(second.location, Point2::new(3.0, 0.0));
        assert_eq!(second.direction, Vector2::new(1.0, 0.0));
        assert_eq!(second.normal, Vector2::new(1.0, 0.0));
        assert!(second.distance.approx_eq_ulps(&3.0, 2));
        assert!(marcher.next().is_none());
    }

    #[test]
    fn intersect_plane_linear() {
        let mut marcher = Hyperplane::intersect_linear(
            &Point2::new(0.0, 0.0),
            &Vector2::new(1.0, 0.0),
            &Vacuum::new(),
            &Hyperplane::new_with_point(
                Vector2::new(-1.0, 0.0),
                &Point2::new(1.0, 0.0)
            ),
            &|_, _| { unimplemented!() }
        );

        let first = marcher.next().unwrap();

        assert_eq!(first.location, Point2::new(1.0, 0.0));
        assert_eq!(first.direction, Vector2::new(1.0, 0.0));
        assert_eq!(first.normal, Vector2::new(-1.0, 0.0));
        assert!(first.distance.approx_eq_ulps(&1.0, 2));
        assert!(marcher.next().is_none());
    }

    #[test]
    fn intersect_halfspace_linear() {
        let mut marcher = HalfSpace::intersect_linear(
            &Point2::new(0.0, 0.0),
            &Vector2::new(1.0, 0.0),
            &Vacuum::new(),
            &HalfSpace::new_with_point(
                Hyperplane::new_with_point(
                    Vector2::new(-1.0, 0.0),
                    &Point2::new(1.0, 0.0)
                ),
                &Point2::new(2.0, 0.0)
            ),
            &|_, _| { unimplemented!() }
        );

        let first = marcher.next().unwrap();

        assert_eq!(first.location, Point2::new(1.0, 0.0));
        assert_eq!(first.direction, Vector2::new(1.0, 0.0));
        assert_eq!(first.normal, Vector2::new(-1.0, 0.0));
        assert!(first.distance.approx_eq_ulps(&1.0, 2));
        assert!(marcher.next().is_none());
    }

    #[test]
    fn intersect_cylinder_linear() {
        let mut marcher = Cylinder::intersect_linear(
            &Point2::new(0.0, 0.0),
            &Vector2::new(1.0, 0.0),
            &Vacuum::new(),
            &Cylinder::new(
                Point2::new(2.0, 0.0),
                &Vector2::new(0.0, 1.0),
                1.0
            ),
            &|_, _| { unimplemented!() }
        );

        let first = marcher.next().unwrap();
        let second = marcher.next().unwrap();

        assert_eq!(first.location, Point2::new(1.0, 0.0));
        assert_eq!(first.direction, Vector2::new(1.0, 0.0));
        assert_eq!(first.normal, Vector2::new(-1.0, 0.0));
        assert!(first.distance.approx_eq_ulps(&1.0, 2));
        assert_eq!(second.location, Point2::new(3.0, 0.0));
        assert_eq!(second.direction, Vector2::new(1.0, 0.0));
        assert_eq!(second.normal, Vector2::new(1.0, 0.0));
        assert!(second.distance.approx_eq_ulps(&3.0, 2));
        assert!(marcher.next().is_none());
    }
}
