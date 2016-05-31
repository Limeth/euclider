use std;
use std::marker::Reflect;
use std::time::Duration;
use std::any::TypeId;
use std::any::Any;
use na;
use na::NumPoint;
use na::NumVector;
use na::PointAsVector;
use image::Rgba;
use SimulationContext;

pub trait Entity<P: NumPoint<f32>> where Self: Sync {
    fn as_updatable_mut(&mut self) -> Option<&mut Updatable<P>>;
    fn as_updatable(&self) -> Option<&Updatable<P>>;
    fn as_traceable_mut(&mut self) -> Option<&mut Traceable<P>>;
    fn as_traceable(&self) -> Option<&Traceable<P>>;
}

pub trait Camera<P: NumPoint<f32>>: Entity<P> {
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

pub struct Intersection<P: NumPoint<f32>> {
    pub location: P,
    pub direction: <P as PointAsVector>::Vector,
    pub distance_squared: f32,
}

#[derive(Copy, Clone)]
pub struct TracingContext<'a, P: 'a + NumPoint<f32>> {
    pub time: &'a Duration,
    pub origin_traceable: &'a Traceable<P>,
    pub intersection_traceable: &'a Traceable<P>,
    pub intersection: &'a Intersection<P>,
    pub trace: &'a Fn(&Duration,
                      &Traceable<P>,
                      &P,
                      &<P as PointAsVector>::Vector) -> Rgba<u8>,
}

pub trait Shape<P: NumPoint<f32>>
    where Self: HasId
{
    fn get_normal_at(&self, point: &P) -> <P as PointAsVector>::Vector;
    fn is_point_inside(&self, point: &P) -> bool;
}

pub trait Material<P: NumPoint<f32>> where Self: HasId {}

pub trait Surface<P: NumPoint<f32>> {
    fn get_color<'a>(&self, context: TracingContext<'a, P>) -> Rgba<u8>;
}

pub trait AbstractSurface<P: NumPoint<f32>> {
    fn get_reflection_ratio(&self, context: &TracingContext<P>) -> f32;
    fn get_reflection_direction(&self, context: &TracingContext<P>) -> <P as PointAsVector>::Vector;
    fn get_surface_color(&self, context: &TracingContext<P>) -> Rgba<u8>;
}

impl<P: NumPoint<f32>, A: AbstractSurface<P>> Surface<P> for A {
    fn get_color<'a>(&self, context: TracingContext<'a, P>) -> Rgba<u8> {
        let reflection_ratio = self.get_reflection_ratio(&context).min(0.0).max(1.0);
        let reflection_color: Rgba<u8>;

        if reflection_ratio == 0.0 {
            return self.get_surface_color(&context);
        } else if reflection_ratio == 1.0 {
            let reflection_direction = self.get_reflection_direction(&context);
            let trace = context.trace;
            let a: P::Vector;
            // let new_origin = context.intersection.location
            //                  + reflection_direction * std::f32::EPSILON;

            return trace(context.time,
                         context.origin_traceable,
                         // &new_origin,
                         &context.intersection.location,
                         &reflection_direction);
        } else {

        }

        Rgba { data: [0; 4] }
    }
}

pub struct ComposableSurface<P: NumPoint<f32>> {
    pub reflection_ratio: fn(&TracingContext<P>) -> f32,
    pub reflection_direction: fn(&TracingContext<P>) -> <P as PointAsVector>::Vector,
    pub surface_color: fn(&TracingContext<P>) -> Rgba<u8>,
}

impl<P: NumPoint<f32>> AbstractSurface<P> for ComposableSurface<P> {
    fn get_reflection_ratio(&self, context: &TracingContext<P>) -> f32 {
        let reflection_ratio = self.reflection_ratio;
        reflection_ratio(context)
    }

    fn get_reflection_direction(&self, context: &TracingContext<P>) -> <P as PointAsVector>::Vector {
        let reflection_direction = self.reflection_direction;
        reflection_direction(context)
    }

    fn get_surface_color(&self, context: &TracingContext<P>) -> Rgba<u8> {
        let surface_color = self.surface_color;
        surface_color(context)
    }
}

pub trait Updatable<P: NumPoint<f32>>: Entity<P> {
    fn update(&mut self, delta_time: &Duration, context: &SimulationContext);
}

pub trait Traceable<P: NumPoint<f32>>: Entity<P> {
    fn trace(&self) -> Rgba<u8>;
    fn shape(&self) -> &Shape<P>;
    fn material(&self) -> &Material<P>;
    fn surface(&self) -> Option<&Surface<P>>;
}

pub trait Locatable<P: NumPoint<f32>> {
    fn location_mut(&mut self) -> &mut P;
    fn location(&self) -> &P;
    fn set_location(&mut self, location: P);
}

pub trait Rotatable<P: NumPoint<f32>> {
    fn rotation_mut(&mut self) -> &mut <P as PointAsVector>::Vector;
    fn rotation(&self) -> &<P as PointAsVector>::Vector;
    fn set_rotation(&mut self, location: <P as PointAsVector>::Vector);
}

// // TODO
// // ((x-m)^2)/(a^2) + ((y-n)^2)/(b^2) = 1
// pub struct Sphere<P: NumPoint<f32>> {
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

impl<P: NumPoint<f32>> Material<P> for Vacuum {}

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

impl<P: NumPoint<f32>> Shape<P> for VoidShape {
    fn get_normal_at(&self, point: &P) -> <P as PointAsVector>::Vector {
        let origin: P = na::origin();
        origin.to_vector()
    }

    fn is_point_inside(&self, point: &P) -> bool {
        true
    }
}

pub struct Void<P: NumPoint<f32>> {
    shape: Box<VoidShape>,
    material: Box<Material<P>>,
}

unsafe impl<P: NumPoint<f32>> Sync for Void<P> {}

impl<P: NumPoint<f32>> Void<P> {
    pub fn new(material: Box<Material<P>>) -> Void<P> {
        Void {
            shape: Box::new(VoidShape::new()),
            material: material,
        }
    }

    pub fn new_with_vacuum() -> Void<P> {
        Self::new(Box::new(Vacuum::new()))
    }
}

impl<P: NumPoint<f32>> Entity<P> for Void<P> {
    fn as_updatable_mut(&mut self) -> Option<&mut Updatable<P>> {
        None
    }

    fn as_updatable(&self) -> Option<&Updatable<P>> {
        None
    }

    fn as_traceable_mut(&mut self) -> Option<&mut Traceable<P>> {
        Some(self)
    }

    fn as_traceable(&self) -> Option<&Traceable<P>> {
        Some(self)
    }
}

impl<P: NumPoint<f32>> Traceable<P> for Void<P> {
    fn trace(&self) -> Rgba<u8> {
        // TODO
        Rgba { data: [0u8, 0u8, 255u8, 255u8] }
    }

    fn shape(&self) -> &Shape<P> {
        self.shape.as_ref()
    }

    fn material(&self) -> &Material<P> {
        self.material.as_ref()
    }

    fn surface(&self) -> Option<&Surface<P>> {
        None
    }
}
