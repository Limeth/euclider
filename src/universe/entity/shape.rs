use std::marker::PhantomData;
use std::fmt;
use std::fmt::Debug;
use std::fmt::Display;
use std::marker::Reflect;
use std::time::Duration;
use std::any::TypeId;
use std::any::Any;
use std::sync::Arc;
use std::iter;
use palette::Rgba;
use universe::entity::Traceable;
use universe::entity::material::Material;
use universe::entity::material::TransitionHandlers;
use universe::entity::material::Vacuum;
use util::CustomFloat;
use util::CustomPoint;
use util::CustomVector;
use util::HasId;
use util::Provider;
use util::TypePairMap;
use mopa;
use na;

/// Ties a `Material` the ray is passing through and a `Shape` the ray is intersecting to a
/// `GeneralIntersector`
pub type GeneralIntersectors<F, P, V> = TypePairMap<Box<GeneralIntersector<F, P, V>>>;

/// Computes the intersections of a ray in a given `Material` with a given `Shape`.
/// The ray is originating in the given `Point` with a direction of the given `Vector`.
// Send + Sync must be at the end of the type alias definition.
pub type GeneralIntersector<F, P, V> = Fn(&P,
                                          &V,
                                          &Material<F, P, V>,
                                          &Shape<F, P, V>,
                                          Intersector<F, P, V>
                                       ) -> Box<IntersectionMarcher<F, P, V>> + Send + Sync;

pub type IntersectionMarcher<F, P, V> = Iterator<Item = Intersection<F, P, V>>;

/// Computes the intersections of a ray in a given `Material` with a given `Shape`
/// with a predefined `Point` of origin and directional `Vector`.
// TODO: It feels wrong to have a type alias to a reference of another type
pub type Intersector<'a, F, P, V> = &'a Fn(&Material<F, P, V>, &Shape<F, P, V>)
                                           -> Provider<Intersection<F, P, V>>;

/// Calls the `trace` method on the current Universe and returns the resulting color.
// TODO: It feels wrong to have a type alias to a reference of another type
pub type Tracer<'a, F, P, V> = &'a Fn(&Duration, &Traceable<F, P, V>, &P, &V) -> Rgba<F>;

pub trait Shape<F: CustomFloat, P: CustomPoint<F, V>, V: CustomVector<F, P>>
    where Self: HasId + Debug + Display + mopa::Any
{
    fn is_point_inside(&self, point: &P) -> bool;
}

// Make sure the type constraints are spaced, so no bitshift operators occur.
mopafy!(Shape< F: CustomFloat, P: CustomPoint< F, V >, V: CustomVector< F, P > >);

#[derive(Debug, Copy, Clone)]
pub struct Intersection<F: CustomFloat, P: CustomPoint<F, V>, V: CustomVector<F, P>> {
    pub location: P,
    pub direction: V,
    pub normal: V,
    pub distance_squared: F,
    pub float_precision: PhantomData<F>,
    pub vector_dimensions: PhantomData<V>,
}

impl<F: CustomFloat, P: CustomPoint<F, V>, V: CustomVector<F, P>> Intersection<F, P, V> {
    pub fn new(location: P, direction: V, normal: V, distance_squared: F) -> Intersection<F, P, V> {
        Intersection {
            location: location,
            direction: direction,
            normal: normal,
            distance_squared: distance_squared,
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
    pub time: &'a Duration,
    pub depth_remaining: &'a u32,
    pub origin_traceable: &'a Traceable<F, P, V>,
    pub intersection_traceable: &'a Traceable<F, P, V>,
    pub intersection: &'a Intersection<F, P, V>,
    pub intersection_normal_closer: &'a V,
    pub exiting: &'a bool,
    pub transitions: &'a TransitionHandlers<F, P, V>,
    pub trace: Tracer<'a, F, P, V>,
}

#[allow(dead_code)]
#[derive(Debug, Copy, Clone)]
pub enum SetOperation {
    Union, // A + B
    Intersection, // A && B
    Complement, // A - B
    SymmetricDifference, // A ^ B
}

struct ComposableShapeIterator<F: CustomFloat, P: CustomPoint<F, V>, V: CustomVector<F, P>> {
    shape_a: Arc<Shape<F, P, V>>,
    shape_b: Arc<Shape<F, P, V>>,
    provider_a: Provider<Intersection<F, P, V>>,
    provider_b: Provider<Intersection<F, P, V>>,
    index_a: usize,
    index_b: usize,
}

impl<F: CustomFloat, P: CustomPoint<F, V>, V: CustomVector<F, P>> ComposableShapeIterator<F, P, V> {
    fn new(shape_a: Arc<Shape<F, P, V>>,
           shape_b: Arc<Shape<F, P, V>>,
           provider_a: Provider<Intersection<F, P, V>>,
           provider_b: Provider<Intersection<F, P, V>>)
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
    fn new(shape_a: Arc<Shape<F, P, V>>,
           shape_b: Arc<Shape<F, P, V>>,
           provider_a: Provider<Intersection<F, P, V>>,
           provider_b: Provider<Intersection<F, P, V>>)
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
            let intersection_a = self.data.provider_a[self.data.index_a];
            let intersection_b = self.data.provider_b[self.data.index_b];

            if intersection_a.is_some() {
                if intersection_b.is_some() {
                    let unwrapped_a = intersection_a.unwrap();
                    let unwrapped_b = intersection_b.unwrap();
                    let closer: Intersection<F, P, V>;
                    let closer_index: &mut usize;
                    let further_shape: &Shape<F, P, V>;

                    if unwrapped_a.distance_squared < unwrapped_b.distance_squared {
                        closer = unwrapped_a;
                        closer_index = &mut self.data.index_a;
                        further_shape = self.data.shape_b.as_ref();
                    } else {
                        closer = unwrapped_b;
                        closer_index = &mut self.data.index_b;
                        further_shape = self.data.shape_a.as_ref();
                    }

                    if !further_shape.is_point_inside(&closer.location) {
                        *closer_index += 1;
                        return Some(closer);
                    }

                    *closer_index += 1;
                } else {
                    self.data.index_a += 1;
                    return intersection_a;
                }
            } else {
                if intersection_b.is_some() {
                    self.data.index_b += 1;
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
    fn new(shape_a: Arc<Shape<F, P, V>>,
           shape_b: Arc<Shape<F, P, V>>,
           provider_a: Provider<Intersection<F, P, V>>,
           provider_b: Provider<Intersection<F, P, V>>)
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
            let intersection_a = self.data.provider_a[self.data.index_a];
            let intersection_b = self.data.provider_b[self.data.index_b];

            if intersection_a.is_some() {
                if intersection_b.is_some() {
                    let unwrapped_a = intersection_a.unwrap();
                    let unwrapped_b = intersection_b.unwrap();
                    let closer: Intersection<F, P, V>;
                    let closer_index: &mut usize;
                    let further_shape: &Shape<F, P, V>;

                    if unwrapped_a.distance_squared < unwrapped_b.distance_squared {
                        closer = unwrapped_a;
                        closer_index = &mut self.data.index_a;
                        further_shape = self.data.shape_b.as_ref();
                    } else {
                        closer = unwrapped_b;
                        closer_index = &mut self.data.index_b;
                        further_shape = self.data.shape_a.as_ref();
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
    fn new(shape_a: Arc<Shape<F, P, V>>,
           shape_b: Arc<Shape<F, P, V>>,
           provider_a: Provider<Intersection<F, P, V>>,
           provider_b: Provider<Intersection<F, P, V>>)
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
            let intersection_a = self.data.provider_a[self.data.index_a];
            let intersection_b = self.data.provider_b[self.data.index_b];

            if intersection_a.is_some() {
                if intersection_b.is_some() {
                    let unwrapped_a = intersection_a.unwrap();
                    let unwrapped_b = intersection_b.unwrap();

                    if unwrapped_a.distance_squared < unwrapped_b.distance_squared {
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
    fn new(shape_a: Arc<Shape<F, P, V>>,
           shape_b: Arc<Shape<F, P, V>>,
           provider_a: Provider<Intersection<F, P, V>>,
           provider_b: Provider<Intersection<F, P, V>>)
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
        loop {
            let intersection_a = self.data.provider_a[self.data.index_a];
            let intersection_b = self.data.provider_b[self.data.index_b];

            if intersection_a.is_some() {
                if intersection_b.is_some() {
                    let unwrapped_a = intersection_a.unwrap();
                    let unwrapped_b = intersection_b.unwrap();
                    let closer: Intersection<F, P, V>;
                    let closer_index: &mut usize;
                    let further_shape: &Shape<F, P, V>;

                    if unwrapped_a.distance_squared < unwrapped_b.distance_squared {
                        closer = unwrapped_a;
                        closer_index = &mut self.data.index_a;
                        further_shape = self.data.shape_b.as_ref();
                    } else {
                        closer = unwrapped_b;
                        closer_index = &mut self.data.index_b;
                        further_shape = self.data.shape_a.as_ref();
                    }

                    if further_shape.is_point_inside(&closer.location) {
                        *closer_index += 1;
                        let mut closer_inverted = closer;
                        closer_inverted.normal = -closer_inverted.normal;
                        return Some(closer_inverted);
                    }

                    *closer_index += 1;
                    return Some(closer);
                } else {
                    let unwrapped_a = intersection_a.unwrap();

                    self.data.index_a += 1;

                    if self.data.shape_b.is_point_inside(&unwrapped_a.location) {
                        let mut closer_inverted = unwrapped_a;
                        closer_inverted.normal = -closer_inverted.normal;
                        return Some(closer_inverted);
                    }

                    return intersection_a;
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

                return intersection_b;
            }
        }
    }
}

impl Display for SetOperation {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

pub struct ComposableShape<F: CustomFloat, P: CustomPoint<F, V>, V: CustomVector<F, P>> {
    pub a: Arc<Shape<F, P, V>>,
    pub b: Arc<Shape<F, P, V>>,
    pub operation: SetOperation,
    pub float_precision: PhantomData<F>,
    pub dimensions: PhantomData<P>,
    pub vector_dimensions: PhantomData<V>,
}

impl<F: CustomFloat, P: CustomPoint<F, V>, V: CustomVector<F, P>> ComposableShape<F, P, V> {
    pub fn new<A: Shape<F, P, V> + 'static, B: Shape<F, P, V> + 'static>
        (a: A,
         b: B,
         operation: SetOperation)
         -> ComposableShape<F, P, V> {
        ComposableShape {
            a: Arc::new(a),
            b: Arc::new(b),
            operation: operation,
            float_precision: PhantomData,
            dimensions: PhantomData,
            vector_dimensions: PhantomData,
        }
    }

    pub fn of<S: Shape<F, P, V> + 'static, I: Iterator<Item = S>>(mut shapes: I,
                                                                  operation: SetOperation)
                                                                  -> ComposableShape<F, P, V> {
        let mut result = ComposableShape {
            a: Arc::new(shapes.next().unwrap()),
            b: Arc::new(shapes.next().unwrap()),
            operation: operation,
            float_precision: PhantomData,
            dimensions: PhantomData,
            vector_dimensions: PhantomData,
        };

        for shape in shapes {
            result.b = Arc::new(ComposableShape {
                a: result.b,
                b: Arc::new(shape),
                operation: operation,
                float_precision: PhantomData,
                dimensions: PhantomData,
                vector_dimensions: PhantomData,
            });
        }

        result
    }

    #[allow(unused_variables)]
    pub fn intersect_in_vacuum(location: &P,
                               direction: &V,
                               vacuum: &Material<F, P, V>,
                               shape: &Shape<F, P, V>,
                               intersect: Intersector<F, P, V>)
                               -> Box<Iterator<Item = Intersection<F, P, V>>> {
        vacuum.as_any().downcast_ref::<Vacuum>().unwrap();
        let composed: &ComposableShape<F, P, V> =
            shape.as_any().downcast_ref::<ComposableShape<F, P, V>>().unwrap();
        let provider_a = intersect(vacuum, composed.a.as_ref());
        let provider_b = intersect(vacuum, composed.b.as_ref());
        match composed.operation {
            SetOperation::Union => {
                Box::new(UnionIterator::new(composed.a.clone(),
                                            composed.b.clone(),
                                            provider_a,
                                            provider_b))
            }
            SetOperation::Intersection => {
                Box::new(IntersectionIterator::new(composed.a.clone(),
                                                   composed.b.clone(),
                                                   provider_a,
                                                   provider_b))
            }
            SetOperation::Complement => {
                Box::new(ComplementIterator::new(composed.a.clone(),
                                                 composed.b.clone(),
                                                 provider_a,
                                                 provider_b))
            }
            SetOperation::SymmetricDifference => {
                Box::new(SymmetricDifferenceIterator::new(composed.a.clone(),
                                                          composed.b.clone(),
                                                          provider_a,
                                                          provider_b))
            }
        }
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

impl<F: CustomFloat, P: CustomPoint<F, V>, V: CustomVector<F, P>> Debug for ComposableShape<F, P, V> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "ComposableShape [ operation: {:?} ]", self.operation)
    }
}

impl<F: CustomFloat, P: CustomPoint<F, V>, V: CustomVector<F, P>> Display for ComposableShape<F,
                                                                                              P,
                                                                                              V> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f,
               "ComposableShape [ a: {}, b: {}, operation: {} ]",
               self.a,
               self.b,
               self.operation)
    }
}

impl<F: CustomFloat, P: CustomPoint<F, V>, V: CustomVector<F, P>> Reflect for ComposableShape<F,
                                                                                              P,
                                                                                              V> {
}

impl<F: 'static + CustomFloat, P: 'static + CustomPoint<F, V>, V: 'static + CustomVector<F, P>> HasId
        for ComposableShape<F, P, V> {
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

#[derive(Default)]
pub struct VoidShape {}

impl VoidShape {
    pub fn new() -> Self {
        VoidShape {}
    }
}

impl HasId for VoidShape {
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

impl<F: CustomFloat, P: CustomPoint<F, V>, V: CustomVector<F, P>> Shape<F, P, V> for VoidShape {
    #[allow(unused_variables)]
    fn is_point_inside(&self, point: &P) -> bool {
        true
    }
}

impl Debug for VoidShape {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "VoidShape")
    }
}

impl Display for VoidShape {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "VoidShape")
    }
}

pub struct Sphere<F: CustomFloat, P: CustomPoint<F, V>, V: CustomVector<F, P>> {
    pub location: P,
    pub radius: F,
    marker_vector: PhantomData<V>,
}

impl<F: CustomFloat, P: CustomPoint<F, V>, V: CustomVector<F, P>> Sphere<F, P, V> {
    pub fn new(location: P, radius: F) -> Sphere<F, P, V> {
        Sphere {
            location: location,
            radius: radius,
            marker_vector: PhantomData,
        }
    }
}

impl<F: CustomFloat, P: CustomPoint<F, V>, V: CustomVector<F, P>> Reflect for Sphere<F, P, V> {}

impl<F: CustomFloat, P: CustomPoint<F, V>, V: CustomVector<F, P>> HasId for Sphere<F, P, V> {
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

impl<F: CustomFloat, P: CustomPoint<F, V>, V: CustomVector<F, P>> Shape<F, P, V> for Sphere<F, P, V> {
    fn is_point_inside(&self, point: &P) -> bool {
        na::distance_squared(&self.location, point) <= self.radius * self.radius
    }
}

impl<F: CustomFloat, P: CustomPoint<F, V>, V: CustomVector<F, P>> Debug for Sphere<F, P, V> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f,
               "Sphere [ location: {:?}, radius: {:?} ]",
               self.location,
               self.radius)
    }
}

impl<F: CustomFloat, P: CustomPoint<F, V>, V: CustomVector<F, P>> Display for Sphere<F, P, V> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Sphere")
    }
}

#[allow(unused_variables)]
pub fn intersect_void<F: CustomFloat, P: CustomPoint<F, V>, V: CustomVector<F, P>>
    (location: &P,
     direction: &V,
     material: &Material<F, P, V>,
     void: &Shape<F, P, V>,
     intersect: Intersector<F, P, V>)
     -> Box<IntersectionMarcher<F, P, V>> {
    void.as_any().downcast_ref::<VoidShape>().unwrap();
    Box::new(iter::empty())
}
