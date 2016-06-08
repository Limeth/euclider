use std; use std::marker::PhantomData;
use std::fmt;
use std::fmt::Debug;
use std::marker::Reflect;
use std::time::Duration;
use std::any::TypeId;
use std::any::Any;
use std::collections::HashMap;
use na;
use na::BaseFloat;
use na::NumPoint;
use na::PointAsVector;
use image::Rgba;
use palette;
use palette::Blend;
use SimulationContext;
use universe::NalgebraOperations;
use util;

pub trait Entity<F: BaseFloat, P: NumPoint<F>> where Self: Sync {
    fn as_updatable_mut(&mut self) -> Option<&mut Updatable<F, P>>;
    fn as_updatable(&self) -> Option<&Updatable<F, P>>;
    fn as_traceable_mut(&mut self) -> Option<&mut Traceable<F, P>>;
    fn as_traceable(&self) -> Option<&Traceable<F, P>>;
}

pub trait Camera<F: BaseFloat, P: NumPoint<F>>: Entity<F, P> {
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

pub struct Intersection<F: BaseFloat, P: NumPoint<F>> {
    pub location: P,
    pub direction: <P as PointAsVector>::Vector,
    pub distance_squared: F,
    float_precision: PhantomData<F>,
}

#[derive(Copy, Clone)]
pub struct TracingContext<'a, F: BaseFloat, P: 'a + NumPoint<F>> {
    pub time: &'a Duration,
    pub origin_traceable: &'a Traceable<F, P>,
    pub intersection_traceable: &'a Traceable<F, P>,
    pub intersection: &'a Intersection<F, P>,
    pub nalgebra_operations: &'a NalgebraOperations<F, P>,
    pub transitions: &'a HashMap<(TypeId, TypeId),
                                     fn(&Material<F, P>,
                                        &Material<F, P>,
                                        &TracingContext<F, P>
                                        ) -> Rgba<u8>>,
    pub trace: &'a Fn(&Duration,
                      &Traceable<F, P>,
                      &P,
                      &<P as PointAsVector>::Vector) -> Rgba<u8>,
}

pub trait Shape<F: BaseFloat, P: NumPoint<F>>
    where Self: HasId
{
    fn get_normal_at(&self, point: &P) -> <P as PointAsVector>::Vector;
    fn is_point_inside(&self, point: &P) -> bool;
}

pub trait Material<F: BaseFloat, P: NumPoint<F>> where Self: HasId + Debug {}

pub trait Surface<F: BaseFloat, P: NumPoint<F>> {
    fn get_color<'a>(&self, context: TracingContext<'a, F, P>) -> Rgba<u8>;
}

pub trait AbstractSurface<F: BaseFloat, P: NumPoint<F>> {
    fn get_reflection_ratio(&self, context: &TracingContext<F, P>) -> F;
    fn get_reflection_direction(&self, context: &TracingContext<F, P>) -> <P as PointAsVector>::Vector;
    fn get_surface_color(&self, context: &TracingContext<F, P>) -> Rgba<u8>;

    fn get_intersection_color(&self, reflection_ratio: F, context: &TracingContext<F, P>) -> Option<Rgba<u8>> {
        if reflection_ratio >= 1.0 {
            return None;
        }

        Some({
            let surface_color = self.get_surface_color(&context);
            let surface_color_alpha = surface_color.data[3];

            if surface_color_alpha == std::u8::MAX {
                surface_color
            } else {
                let origin_material = context.origin_traceable.material();
                let intersection_material = context.intersection_traceable.material();
                let transition = context.transitions.get(&(origin_material.id(),
                                                           intersection_material.id()))
                                                    .expect(&format!("No transition found from material {:?} to material {:?}. Make sure to register one.", origin_material, intersection_material));
                let transition_color = transition(origin_material,
                                                  intersection_material,
                                                  &context);
                let surface_palette: palette::Rgba<F> = palette::Rgba::new_u8(surface_color[0],
                                                                                surface_color[1],
                                                                                surface_color[2],
                                                                                surface_color[3]);
                let transition_palette = palette::Rgba::new_u8(transition_color[0],
                                                               transition_color[1],
                                                               transition_color[2],
                                                               transition_color[3]);
                let result = surface_palette.plus(transition_palette).to_pixel();

                Rgba {
                    data: result
                }
            }
        })
    }

    fn get_reflection_color(&self, reflection_ratio: F, context: &TracingContext<F, P>) -> Option<Rgba<u8>> {
        if reflection_ratio <= 0.0 {
            return None;
        }

        let normal = context.intersection_traceable.shape().get_normal_at(&context.intersection.location);

        // Is the directional vector pointing away from the shape, or is it going inside?
        if context.nalgebra_operations.dot(
            &context.intersection.direction,
            &normal
            ) > 0.0 {
            return None;
        } else {
            let reflection_direction = self.get_reflection_direction(&context);
            let trace = context.trace;
            // // Offset the new origin, so it doesn't hit the same shape over and over
            // let vector_to_point = context.vector_to_point;
            // let new_origin = context.intersection.location
            //                  + (vector_to_point(&reflection_direction) * std::F::EPSILON * 8.0)
            //                     .to_vector();

            return Some(trace(context.time,
                  context.origin_traceable,
                  &context.intersection.location,
                  // &new_origin,
                  &reflection_direction));
        }
    }
}

impl<F: BaseFloat, P: NumPoint<F>, A: AbstractSurface<F, P>> Surface<F, P> for A {
    fn get_color(&self, context: TracingContext<F, P>) -> Rgba<u8> {
        let reflection_ratio = self.get_reflection_ratio(&context).min(1.0).max(0.0);
        let intersection_color: Option<Rgba<u8>> = self.get_intersection_color(reflection_ratio, &context);
        let reflection_color: Option<Rgba<u8>> = self.get_reflection_color(reflection_ratio, &context);
        if intersection_color.is_none() {
            return reflection_color.expect("No intersection color calculated; the reflection color should exist.");
        } else if reflection_color.is_none() {
            return intersection_color.expect("No reflection color calculated; the intersection color should exist.");
        }

        util::combine_color(reflection_color.unwrap(),
                           intersection_color.unwrap(),
                           reflection_ratio)
    }
}

pub struct ComposableSurface<F: BaseFloat, P: NumPoint<F>> {
    pub reflection_ratio: fn(&TracingContext<F, P>) -> F,
    pub reflection_direction: fn(&TracingContext<F, P>) -> <P as PointAsVector>::Vector,
    pub surface_color: fn(&TracingContext<F, P>) -> Rgba<u8>,
}

impl<F: BaseFloat, P: NumPoint<F>> AbstractSurface<F, P> for ComposableSurface<F, P> {
    fn get_reflection_ratio(&self, context: &TracingContext<F, P>) -> F {
        let reflection_ratio = self.reflection_ratio;
        reflection_ratio(context)
    }

    fn get_reflection_direction(&self, context: &TracingContext<F, P>) -> <P as PointAsVector>::Vector {
        let reflection_direction = self.reflection_direction;
        reflection_direction(context)
    }

    fn get_surface_color(&self, context: &TracingContext<F, P>) -> Rgba<u8> {
        let surface_color = self.surface_color;
        surface_color(context)
    }
}

pub trait Updatable<F: BaseFloat, P: NumPoint<F>>: Entity<F, P> {
    fn update(&mut self, delta_time: &Duration, context: &SimulationContext);
}

pub trait Traceable<F: BaseFloat, P: NumPoint<F>>: Entity<F, P> {
    fn trace(&self) -> Rgba<u8>;
    fn shape(&self) -> &Shape<F, P>;
    fn material(&self) -> &Material<F, P>;
    fn surface(&self) -> Option<&Surface<F, P>>;
}

pub trait Locatable<F: BaseFloat, P: NumPoint<F>> {
    fn location_mut(&mut self) -> &mut P;
    fn location(&self) -> &P;
    fn set_location(&mut self, location: P);
}

pub trait Rotatable<F: BaseFloat, P: NumPoint<F>> {
    fn rotation_mut(&mut self) -> &mut <P as PointAsVector>::Vector;
    fn rotation(&self) -> &<P as PointAsVector>::Vector;
    fn set_rotation(&mut self, location: <P as PointAsVector>::Vector);
}

// // TODO
// // ((x-m)^2)/(a^2) + ((y-n)^2)/(b^2) = 1
// pub struct Sphere<P: NumPoint<F>> {
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

impl<F: BaseFloat, P: NumPoint<F>> Material<F, P> for Vacuum {}

impl Debug for Vacuum {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Material [ Vacuum ]")
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

impl<F: BaseFloat, P: NumPoint<F>> Shape<F, P> for VoidShape {
    fn get_normal_at(&self, point: &P) -> <P as PointAsVector>::Vector {
        let origin: P = na::origin();
        origin.to_vector()
    }

    fn is_point_inside(&self, point: &P) -> bool {
        true
    }
}

pub struct Void<F: BaseFloat, P: NumPoint<F>> {
    shape: Box<VoidShape>,
    material: Box<Material<F, P>>,
}

unsafe impl<F: BaseFloat, P: NumPoint<F>> Sync for Void<F, P> {}

impl<F: BaseFloat, P: NumPoint<F>> Void<F, P> {
    pub fn new(material: Box<Material<F, P>>) -> Void<F, P> {
        Void {
            shape: Box::new(VoidShape::new()),
            material: material,
        }
    }

    pub fn new_with_vacuum() -> Void<F, P> {
        Self::new(Box::new(Vacuum::new()))
    }
}

impl<F: BaseFloat, P: NumPoint<F>> Entity<F, P> for Void<F, P> {
    fn as_updatable_mut(&mut self) -> Option<&mut Updatable<F, P>> {
        None
    }

    fn as_updatable(&self) -> Option<&Updatable<F, P>> {
        None
    }

    fn as_traceable_mut(&mut self) -> Option<&mut Traceable<F, P>> {
        Some(self)
    }

    fn as_traceable(&self) -> Option<&Traceable<F, P>> {
        Some(self)
    }
}

impl<F: BaseFloat, P: NumPoint<F>> Traceable<F, P> for Void<F, P> {
    fn trace(&self) -> Rgba<u8> {
        // TODO
        Rgba { data: [0u8, 0u8, 255u8, 255u8] }
    }

    fn shape(&self) -> &Shape<F, P> {
        self.shape.as_ref()
    }

    fn material(&self) -> &Material<F, P> {
        self.material.as_ref()
    }

    fn surface(&self) -> Option<&Surface<F, P>> {
        None
    }
}
