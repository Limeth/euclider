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

pub trait HasId {
    fn id_static() -> TypeId
        where Self: Sized + Reflect + 'static
    {
        TypeId::of::<Self>()
    }

    fn id(&self) -> TypeId;
    fn as_any(&self) -> &Any;
    fn as_any_mut(&mut self) -> &mut Any;
}

pub struct Intersection<F: CustomFloat, P: CustomPoint<F, V>, V: CustomVector<F, P>> {
    pub location: P,
    pub direction: <P as PointAsVector>::Vector,
    pub normal: <P as PointAsVector>::Vector,
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
    fn get_color<'a>(&self, context: TracingContext<'a, F, P, V>) -> Rgba<F>;
}

pub struct ComposableShape<F: CustomFloat, P: CustomPoint<F, V>, V: CustomVector<F, P>,
        A: Shape<F, P, V>, B: Shape<F, P, V>> {
    pub a: A,
    pub b: B,
    pub operation: SetOperation,
    pub float_precision: PhantomData<F>,
    pub dimensions: PhantomData<P>,
    pub vector_dimensions: PhantomData<V>,
}

#[allow(dead_code)]
#[derive(Debug)]
pub enum SetOperation {
    Union, // A + B
    Intersection,  // A && B
    Complement,  // A - B
    SymmetricDifference,  // A ^ B
}

impl Display for SetOperation {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl<F: CustomFloat, P: CustomPoint<F, V>, V: CustomVector<F, P>, A: Shape<F, P, V>, B: Shape<F, P, V>>
        ComposableShape<F, P, V, A, B> {
    pub fn new(a: A, b: B, operation: SetOperation) -> ComposableShape<F, P, V, A, B> {
        ComposableShape {
            a: a,
            b: b,
            operation: operation,
            float_precision: PhantomData,
            dimensions: PhantomData,
            vector_dimensions: PhantomData,
        }
    }
}

impl<F: 'static + CustomFloat, P: 'static + CustomPoint<F, V>, V: 'static + CustomVector<F, P>, A: 'static + Shape<F, P, V>, B: 'static + Shape<F, P, V>> Shape<F, P, V>
        for ComposableShape<F, P, V, A, B> {
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

impl<F: CustomFloat, P: CustomPoint<F, V>, V: CustomVector<F, P>, A: Shape<F, P, V>, B: Shape<F, P, V>> Debug
        for ComposableShape<F, P, V, A, B> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "ComposableShape [ operation: {:?} ]", self.operation)
    }
}

impl<F: CustomFloat, P: CustomPoint<F, V>, V: CustomVector<F, P>, A: Shape<F, P, V>, B: Shape<F, P, V>> Display
        for ComposableShape<F, P, V, A, B> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "ComposableShape [ a: {}, b: {}, operation: {} ]", self.a, self.b,
               self.operation)
    }
}

impl<F: CustomFloat, P: CustomPoint<F, V>, V: CustomVector<F, P>, A: Shape<F, P, V>, B: Shape<F, P, V>> Reflect
        for ComposableShape<F, P, V, A, B> {}

impl<F: 'static + CustomFloat, P: 'static + CustomPoint<F, V>, V: 'static + CustomVector<F, P>, A: 'static + Shape<F, P, V>, B: 'static + Shape<F, P, V>> HasId
        for ComposableShape<F, P, V, A, B> {
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
            let surface_color = self.get_surface_color(&context);
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
                let transition_color = transition(origin_material, intersection_material, &context);
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

        let reflection_direction = self.get_reflection_direction(&context);
        let trace = context.trace;
        // // Offset the new origin, so it doesn't hit the same shape over and over
        // let vector_to_point = context.vector_to_point;
        // let new_origin = context.intersection.location
        //                  + (vector_to_point(&reflection_direction) * std::F::EPSILON * 8.0)
        //                     .to_vector();

        return trace(context.time,
                     context.origin_traceable,
                     &context.intersection.location,
                     // &new_origin,
                     &reflection_direction);
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
