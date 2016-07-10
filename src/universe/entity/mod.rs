use std;
use std::marker::PhantomData;
use std::fmt;
use std::fmt::Debug;
use std::fmt::Display;
use std::marker::Reflect;
use std::time::Duration;
use std::any::TypeId;
use std::any::Any;
use std::collections::HashMap;
use std::sync::Arc;
use num::traits::NumCast;
use na::PointAsVector;
use palette;
use palette::Rgba;
use palette::Blend;
use SimulationContext;
use util;
use util::CustomFloat;
use util::CustomPoint;
use util::CustomVector;
use util::HasId;
use util::Provider;

pub trait Entity<F: CustomFloat, P: CustomPoint<F, V>, V: CustomVector<F, P>>
    where Self: Sync
{
    fn as_updatable_mut(&mut self) -> Option<&mut Updatable<F, P, V>>;
    fn as_updatable(&self) -> Option<&Updatable<F, P, V>>;
    fn as_traceable_mut(&mut self) -> Option<&mut Traceable<F, P, V>>;
    fn as_traceable(&self) -> Option<&Traceable<F, P, V>>;
}

pub trait Camera<F: CustomFloat, P: CustomPoint<F, V>, V: CustomVector<F, P>>: Entity<F, P, V> {
    fn get_ray_point(&self,
                     screen_x: i32,
                     screen_y: i32,
                     screen_width: i32,
                     screen_height: i32)
                     -> P;
    fn get_ray_vector(&self,
                      screen_x: i32,
                      screen_y: i32,
                      screen_width: i32,
                      screen_height: i32)
                      -> <P as PointAsVector>::Vector;
    fn max_depth(&self) -> u32;
}

pub type Intersector<'a, F, P, V> = &'a Fn(
                                       &Material<F, P, V>,
                                       &Shape<F, P, V>
                                    ) -> Provider<Intersection<F, P, V>>;

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
    pub fn new(location: P, direction: <P as PointAsVector>::Vector,
               normal: <P as PointAsVector>::Vector,
               distance_squared: F)
               -> Intersection<F, P, V> {
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
                          V: 'a + CustomVector<F, P>> {
    pub time: &'a Duration,
    pub depth_remaining: &'a u32,
    pub origin_traceable: &'a Traceable<F, P, V>,
    pub intersection_traceable: &'a Traceable<F, P, V>,
    pub intersection: &'a Intersection<F, P, V>,
    pub intersection_normal_closer: &'a <P as PointAsVector>::Vector,
    pub exiting: &'a bool,
    pub transitions: &'a HashMap<(TypeId, TypeId),
                                 fn(&Material<F, P, V>, &Material<F, P, V>, &TracingContext<F, P, V>)
                                    -> Option<Rgba<F>>>,
    pub trace: &'a Fn(&Duration,
                      &Traceable<F, P, V>,
                      &P,
                      &<P as PointAsVector>::Vector)
                      -> Option<Rgba<F>>,
}

pub trait Shape<F: CustomFloat, P: CustomPoint<F, V>, V: CustomVector<F, P>>
    where Self: HasId + Debug + Display
{
    fn is_point_inside(&self, point: &P) -> bool;
}

pub trait Material<F: CustomFloat, P: CustomPoint<F, V>, V: CustomVector<F, P>> where Self: HasId + Debug + Display {}

pub trait Surface<F: CustomFloat, P: CustomPoint<F, V>, V: CustomVector<F, P>> {
    fn get_color(&self, context: TracingContext<F, P, V>) -> Rgba<F>;
}

#[allow(dead_code)]
#[derive(Debug, Copy, Clone)]
pub enum SetOperation {
    Union, // A + B
    Intersection,  // A && B
    Complement,  // A - B
    SymmetricDifference,  // A ^ B
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
           provider_b: Provider<Intersection<F, P, V>>) -> ComposableShapeIterator<F, P, V> {
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
           provider_b: Provider<Intersection<F, P, V>>) -> UnionIterator<F, P, V> {
        UnionIterator {
            data: ComposableShapeIterator::new(shape_a, shape_b, provider_a, provider_b),
        }
    }
}

impl<F: CustomFloat, P: CustomPoint<F, V>, V: CustomVector<F, P>>
        Iterator for UnionIterator<F, P, V> {
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
           provider_b: Provider<Intersection<F, P, V>>) -> IntersectionIterator<F, P, V> {
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
           provider_b: Provider<Intersection<F, P, V>>) -> ComplementIterator<F, P, V> {
        ComplementIterator {
            data: ComposableShapeIterator::new(shape_a, shape_b, provider_a, provider_b),
        }
    }
}

impl<F: CustomFloat, P: CustomPoint<F, V>, V: CustomVector<F, P>>
        Iterator for ComplementIterator<F, P, V> {
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

impl<F: CustomFloat, P: CustomPoint<F, V>, V: CustomVector<F, P>> SymmetricDifferenceIterator<F, P, V> {
    fn new(shape_a: Arc<Shape<F, P, V>>,
           shape_b: Arc<Shape<F, P, V>>,
           provider_a: Provider<Intersection<F, P, V>>,
           provider_b: Provider<Intersection<F, P, V>>) -> SymmetricDifferenceIterator<F, P, V> {
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
    pub fn new<A: Shape<F, P, V> + 'static, B: Shape<F, P, V> + 'static>(a: A, b: B, operation: SetOperation) -> ComposableShape<F, P, V> {
        ComposableShape {
            a: Arc::new(a),
            b: Arc::new(b),
            operation: operation,
            float_precision: PhantomData,
            dimensions: PhantomData,
            vector_dimensions: PhantomData,
        }
    }

    pub fn of<S: Shape<F, P, V> + 'static, I: Iterator<Item=S>>(mut shapes: I, operation: SetOperation)
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
                               intersect: &Intersector<F, P, V>)
                               -> Box<Iterator<Item=Intersection<F, P, V>>> {
        vacuum.as_any().downcast_ref::<Vacuum>().unwrap();
        let composed: &ComposableShape<F, P, V> = shape.as_any().downcast_ref::<ComposableShape<F, P, V>>().unwrap();
        let provider_a = intersect(vacuum, composed.a.as_ref());
        let provider_b = intersect(vacuum, composed.b.as_ref());
        match composed.operation {
            SetOperation::Union => {
                Box::new(
                    UnionIterator::new(
                        composed.a.clone(),
                        composed.b.clone(),
                        provider_a,
                        provider_b
                    )
                )
            }
            SetOperation::Intersection => {
                Box::new(
                    IntersectionIterator::new(
                        composed.a.clone(),
                        composed.b.clone(),
                        provider_a,
                        provider_b
                    )
                )
            }
            SetOperation::Complement => {
                Box::new(
                    ComplementIterator::new(
                        composed.a.clone(),
                        composed.b.clone(),
                        provider_a,
                        provider_b
                    )
                )
            }
            SetOperation::SymmetricDifference => {
                Box::new(
                    SymmetricDifferenceIterator::new(
                        composed.a.clone(),
                        composed.b.clone(),
                        provider_a,
                        provider_b
                    )
                )
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

impl<F: CustomFloat, P: CustomPoint<F, V>, V: CustomVector<F, P>> Debug
        for ComposableShape<F, P, V> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "ComposableShape [ operation: {:?} ]", self.operation)
    }
}

impl<F: CustomFloat, P: CustomPoint<F, V>, V: CustomVector<F, P>> Display
        for ComposableShape<F, P, V> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "ComposableShape [ a: {}, b: {}, operation: {} ]", self.a, self.b,
               self.operation)
    }
}

impl<F: CustomFloat, P: CustomPoint<F, V>, V: CustomVector<F, P>> Reflect
        for ComposableShape<F, P, V> {}

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

// pub trait AbstractSurface<F: CustomFloat, P: CustomPoint<F, V>, V: CustomVector<F, P>> {
//     fn get_reflection_ratio(&self, context: &TracingContext<F, P>) -> F;
//     fn get_reflection_direction(&self, context: &TracingContext<F, P>) -> <P as PointAsVector>::Vector;
//     fn get_surface_color(&self, context: &TracingContext<F, P>) -> Rgba<F>;

//     fn get_intersection_color(&self, reflection_ratio: F, context: &TracingContext<F, P>) -> Option<Rgba<F>> {
//         if reflection_ratio >= 1.0 {
//             return None;
//         }

//         Some({
//             let surface_color = self.get_surface_color(&context);
//             let surface_color_alpha = surface_color.data[3];

//             if surface_color_alpha == std::u8::MAX {
//                 surface_color
//             } else {
//                 let origin_material = context.origin_traceable.material();
//                 let intersection_material = context.intersection_traceable.material();
//                 let transition = context.transitions.get(&(origin_material.id(),
//                                                            intersection_material.id()))
//                                                     .expect(&format!("No transition found from material {:?} to material {:?}. Make sure to register one.", origin_material, intersection_material));
//                 let transition_color = transition(origin_material,
//                                                   intersection_material,
//                                                   &context);
//                 let surface_palette: palette::Rgba<F> = palette::Rgba::new_u8(surface_color[0],
//                                                                                 surface_color[1],
//                                                                                 surface_color[2],
//                                                                                 surface_color[3]);
//                 let transition_palette = palette::Rgba::new_u8(transition_color[0],
//                                                                transition_color[1],
//                                                                transition_color[2],
//                                                                transition_color[3]);
//                 let result = surface_palette.plus(transition_palette).to_pixel();

//                 Rgba {
//                     data: result
//                 }
//             }
//         })
//     }

//     fn get_reflection_color(&self, reflection_ratio: F, context: &TracingContext<F, P>) -> Option<Rgba<F>> {
//         if reflection_ratio <= 0.0 {
//             return None;
//         }

//         let normal = context.intersection_traceable.shape().get_normal_at(&context.intersection.location);

//         // Is the directional vector pointing away from the shape, or is it going inside?
//         if context.nalgebra_operations.dot(
//             &context.intersection.direction,
//             &normal
//             ) > 0.0 {
//             return None;
//         } else {
//             let reflection_direction = self.get_reflection_direction(&context);
//             let trace = context.trace;
//             // // Offset the new origin, so it doesn't hit the same shape over and over
//             // let vector_to_point = context.vector_to_point;
//             // let new_origin = context.intersection.location
//             //                  + (vector_to_point(&reflection_direction) * std::F::EPSILON * 8.0)
//             //                     .to_vector();

//             return Some(trace(context.time,
//                   context.origin_traceable,
//                   &context.intersection.location,
//                   // &new_origin,
//                   &reflection_direction));
//         }
//     }
// }

// impl<F: CustomFloat, P: CustomPoint<F, V>, V: CustomVector<F, P>, A: AbstractSurface<F, P>> Surface<F, P> for A {
//     fn get_color(&self, context: TracingContext<F, P>) -> Rgba<F> {
//         let reflection_ratio = self.get_reflection_ratio(&context).min(1.0).max(0.0);
//         let intersection_color: Option<Rgba<F>> = self.get_intersection_color(reflection_ratio, &context);
//         let reflection_color: Option<Rgba<F>> = self.get_reflection_color(reflection_ratio, &context);
//         if intersection_color.is_none() {
//             return reflection_color.expect("No intersection color calculated; the reflection color should exist.");
//         } else if reflection_color.is_none() {
//             return intersection_color.expect("No reflection color calculated; the intersection color should exist.");
//         }

//         util::combine_color(reflection_color.unwrap(),
//                            intersection_color.unwrap(),
//                            reflection_ratio)
//     }
// }

pub struct ComposableSurface<F: CustomFloat, P: CustomPoint<F, V>, V: CustomVector<F, P>> {
    pub reflection_ratio: fn(&TracingContext<F, P, V>) -> F,
    pub reflection_direction: fn(&TracingContext<F, P, V>) -> <P as PointAsVector>::Vector,
    pub surface_color: fn(&TracingContext<F, P, V>) -> Rgba<F>,
}

// impl<F: CustomFloat, P: CustomPoint<F, V>, V: CustomVector<F, P>> AbstractSurface<F, P> for ComposableSurface<F, P> {
//     fn get_reflection_ratio(&self, context: &TracingContext<F, P>) -> F {
//         let reflection_ratio = self.reflection_ratio;
//         reflection_ratio(context)
//     }

//     fn get_reflection_direction(&self, context: &TracingContext<F, P>) -> <P as PointAsVector>::Vector {
//         let reflection_direction = self.reflection_direction;
//         reflection_direction(context)
//     }

//     fn get_surface_color(&self, context: &TracingContext<F, P>) -> Rgba<F> {
//         let surface_color = self.surface_color;
//         surface_color(context)
//     }
// }

impl<F: CustomFloat, P: CustomPoint<F, V>, V: CustomVector<F, P>> ComposableSurface<F, P, V> {
    fn get_reflection_ratio(&self, context: &TracingContext<F, P, V>) -> F {
        let reflection_ratio = self.reflection_ratio;
        reflection_ratio(context)
    }

    fn get_reflection_direction(&self,
                                context: &TracingContext<F, P, V>)
                                -> <P as PointAsVector>::Vector {
        let reflection_direction = self.reflection_direction;
        reflection_direction(context)
    }

    fn get_surface_color(&self, context: &TracingContext<F, P, V>) -> Rgba<F> {
        let surface_color = self.surface_color;
        surface_color(context)
    }

    fn get_intersection_color(&self,
                              reflection_ratio: F,
                              context: &TracingContext<F, P, V>)
                              -> Option<Rgba<F>> {
        if reflection_ratio >= <F as NumCast>::from(1.0).unwrap() {
            return None;
        }

        Some({
            let surface_color = self.get_surface_color(context);
            let surface_color_data: [u8; 4] = surface_color.to_pixel();
            let surface_color_alpha = surface_color_data[3];

            if surface_color_alpha == std::u8::MAX {
                surface_color
            } else {
                let origin_material = context.origin_traceable.material();
                let intersection_material = context.intersection_traceable.material();
                let transition = context.transitions
                    .get(&(origin_material.id(), intersection_material.id()))
                    .expect(&format!("No transition found from material {:?} to material {:?}. \
                                      Make sure to register one.",
                                     origin_material,
                                     intersection_material));
                let transition_color = transition(origin_material, intersection_material, context);
                let surface_palette: Rgba<F> = palette::Rgba::new_u8(surface_color_data[0],
                                                                     surface_color_data[1],
                                                                     surface_color_data[2],
                                                                     surface_color_data[3]);
                let transition_palette = if transition_color.is_some() {
                    let transition_color: [u8; 4] = transition_color.unwrap().to_pixel();

                    palette::Rgba::new_u8(transition_color[0],
                                          transition_color[1],
                                          transition_color[2],
                                          transition_color[3])
                } else {
                    palette::Rgba::new_u8(0, 0, 0, 0)
                };

                surface_palette.plus(transition_palette)
            }
        })
    }

    fn get_reflection_color(&self,
                            reflection_ratio: F,
                            context: &TracingContext<F, P, V>)
                            -> Option<Rgba<F>> {
        if reflection_ratio <= <F as NumCast>::from(0.0).unwrap() {
            return None;
        }

        let reflection_direction = self.get_reflection_direction(context);
        let trace = context.trace;
        // // Offset the new origin, so it doesn't hit the same shape over and over
        // let vector_to_point = context.vector_to_point;
        // let new_origin = context.intersection.location
        //                  + (vector_to_point(&reflection_direction) * std::F::EPSILON * 8.0)
        //                     .to_vector();

        trace(context.time,
              context.origin_traceable,
              &context.intersection.location,
              // &new_origin,
              &reflection_direction)
    }
}

impl<F: CustomFloat, P: CustomPoint<F, V>, V: CustomVector<F, P>> Surface<F, P, V>
        for ComposableSurface<F, P, V> {
    fn get_color(&self, context: TracingContext<F, P, V>) -> Rgba<F> {
        let reflection_ratio = self.get_reflection_ratio(&context)
            .min(<F as NumCast>::from(1.0).unwrap())
            .max(<F as NumCast>::from(0.0).unwrap());
        let intersection_color: Option<Rgba<F>> =
            self.get_intersection_color(reflection_ratio, &context);
        let reflection_color: Option<Rgba<F>> =
            self.get_reflection_color(reflection_ratio, &context);

        if intersection_color.is_none() {
            return reflection_color.expect("No intersection color calculated; the reflection color should exist.");
        } else if reflection_color.is_none() {
            return intersection_color.expect("No reflection color calculated; the intersection color should exist.");
        }

        util::combine_palette_color(reflection_color.unwrap(),
                                    intersection_color.unwrap(),
                                    reflection_ratio)
    }
}

pub trait Updatable<F: CustomFloat, P: CustomPoint<F, V>, V: CustomVector<F, P>>: Entity<F, P, V> {
    fn update(&mut self, delta_time: &Duration, context: &SimulationContext);
}

pub trait Traceable<F: CustomFloat, P: CustomPoint<F, V>, V: CustomVector<F, P>>: Entity<F, P, V> {
    fn shape(&self) -> &Shape<F, P, V>;
    fn material(&self) -> &Material<F, P, V>;
    fn surface(&self) -> Option<&Surface<F, P, V>>;
}

pub trait Locatable<F: CustomFloat, P: CustomPoint<F, V>, V: CustomVector<F, P>> {
    fn location_mut(&mut self) -> &mut P;
    fn location(&self) -> &P;
    fn set_location(&mut self, location: P);
}

pub trait Rotatable<F: CustomFloat, P: CustomPoint<F, V>, V: CustomVector<F, P>> {
    fn rotation_mut(&mut self) -> &mut <P as PointAsVector>::Vector;
    fn rotation(&self) -> &<P as PointAsVector>::Vector;
    fn set_rotation(&mut self, location: <P as PointAsVector>::Vector);
}

// // TODO
// // ((x-m)^2)/(a^2) + ((y-n)^2)/(b^2) = 1
// pub struct Sphere<P: CustomPoint<F, V>, V: CustomVector<F, P>> {
//     location: P, // m/n/o...
//     radii: V, // a/b/c...
// }

pub struct Vacuum {

}

impl Vacuum {
    pub fn new() -> Vacuum {
        Vacuum {}
    }
}

impl HasId for Vacuum {
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

impl<F: CustomFloat, P: CustomPoint<F, V>, V: CustomVector<F, P>> Material<F, P, V> for Vacuum {}

impl Debug for Vacuum {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Vacuum")
    }
}

impl Display for Vacuum {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Vacuum")
    }
}

pub struct VoidShape {}

impl VoidShape {
    pub fn new() -> VoidShape {
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

pub struct Void<F: CustomFloat, P: CustomPoint<F, V>, V: CustomVector<F, P>> {
    shape: Box<VoidShape>,
    material: Box<Material<F, P, V>>,
}

unsafe impl<F: CustomFloat, P: CustomPoint<F, V>, V: CustomVector<F, P>> Sync for Void<F, P, V> {}

impl<F: CustomFloat, P: CustomPoint<F, V>, V: CustomVector<F, P>> Void<F, P, V> {
    pub fn new(material: Box<Material<F, P, V>>) -> Void<F, P, V> {
        Void {
            shape: Box::new(VoidShape::new()),
            material: material,
        }
    }

    pub fn new_with_vacuum() -> Void<F, P, V> {
        Self::new(Box::new(Vacuum::new()))
    }
}

impl<F: CustomFloat, P: CustomPoint<F, V>, V: CustomVector<F, P>> Entity<F, P, V> for Void<F, P, V> {
    fn as_updatable_mut(&mut self) -> Option<&mut Updatable<F, P, V>> {
        None
    }

    fn as_updatable(&self) -> Option<&Updatable<F, P, V>> {
        None
    }

    fn as_traceable_mut(&mut self) -> Option<&mut Traceable<F, P, V>> {
        Some(self)
    }

    fn as_traceable(&self) -> Option<&Traceable<F, P, V>> {
        Some(self)
    }
}

impl<F: CustomFloat, P: CustomPoint<F, V>, V: CustomVector<F, P>> Traceable<F, P, V> for Void<F, P, V> {
    fn shape(&self) -> &Shape<F, P, V> {
        self.shape.as_ref()
    }

    fn material(&self) -> &Material<F, P, V> {
        self.material.as_ref()
    }

    fn surface(&self) -> Option<&Surface<F, P, V>> {
        None
    }
}
