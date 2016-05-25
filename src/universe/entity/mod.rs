extern crate nalgebra as na;
extern crate image;

use std::marker::Reflect;
use std::time::Duration;
use std::any::TypeId;
use std::any::Any;
use self::na::NumPoint;
use self::na::NumVector;
use self::image::Rgba;
use SimulationContext;

pub trait Entity<P: NumPoint<f32>, V: NumVector<f32>> {
    fn as_updatable_mut(&mut self) -> Option<&mut Updatable>;
    fn as_updatable(&self) -> Option<&Updatable>;
    fn as_traceable_mut(&mut self) -> Option<&mut Traceable<P, V>>;
    fn as_traceable(&self) -> Option<&Traceable<P, V>>;
}

pub trait Camera<P: NumPoint<f32>, V: NumVector<f32>>: Entity<P, V> {
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
                      -> V;
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

pub trait Shape<P: NumPoint<f32>, V: NumVector<f32>>
    where Self: HasId
{
    fn get_normal_at(&self, point: &P) -> V;
    fn is_point_inside(&self, point: &P) -> bool;
}

pub trait Material<P: NumPoint<f32>, V: NumVector<f32>> where Self: HasId {}

pub trait Surface<P: NumPoint<f32>, V: NumVector<f32>> where Self: HasId {}

pub trait Updatable {
    fn update(&mut self, delta_time: &Duration, context: &SimulationContext);
}

pub trait Traceable<P: NumPoint<f32>, V: NumVector<f32>> {
    fn trace(&self) -> Rgba<u8>;
    fn shape(&self) -> &Shape<P, V>;
    fn material(&self) -> &Material<P, V>;
    fn surface(&self) -> Option<&Surface<P, V>>;
}

pub trait Locatable<P: NumPoint<f32>> {
    fn location_mut(&mut self) -> &mut P;
    fn location(&self) -> &P;
    fn set_location(&mut self, location: P);
}

pub trait Rotatable<P: NumVector<f32>> {
    fn rotation_mut(&mut self) -> &mut P;
    fn rotation(&self) -> &P;
    fn set_rotation(&mut self, location: P);
}

// // TODO
// // ((x-m)^2)/(a^2) + ((y-n)^2)/(b^2) = 1
// pub struct Sphere<P: NumPoint<f32>, V: NumVector<f32>> {
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

impl<P: NumPoint<f32>, V: NumVector<f32>> Material<P, V> for Vacuum {}

struct VoidShape {}

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

impl<P: NumPoint<f32>, V: NumVector<f32>> Shape<P, V> for VoidShape {
    fn get_normal_at(&self, point: &P) -> V {
        na::zero()
    }

    fn is_point_inside(&self, point: &P) -> bool {
        true
    }
}

pub struct Void<P: NumPoint<f32>, V: NumVector<f32>> {
    shape: Box<VoidShape>,
    material: Box<Material<P, V>>,
}

impl<P: NumPoint<f32>, V: NumVector<f32>> Void<P, V> {
    pub fn new(material: Box<Material<P, V>>) -> Void<P, V> {
        Void {
            shape: Box::new(VoidShape::new()),
            material: material,
        }
    }

    pub fn new_with_vacuum() -> Void<P, V> {
        Self::new(Box::new(Vacuum::new()))
    }
}

impl<P: NumPoint<f32>, V: NumVector<f32>> Entity<P, V> for Void<P, V> {
    fn as_updatable_mut(&mut self) -> Option<&mut Updatable> {
        None
    }

    fn as_updatable(&self) -> Option<&Updatable> {
        None
    }

    fn as_traceable_mut(&mut self) -> Option<&mut Traceable<P, V>> {
        Some(self)
    }

    fn as_traceable(&self) -> Option<&Traceable<P, V>> {
        Some(self)
    }
}

impl<P: NumPoint<f32>, V: NumVector<f32>> Traceable<P, V> for Void<P, V> {
    fn trace(&self) -> Rgba<u8> {
        // TODO
        Rgba { data: [0u8, 0u8, 255u8, 255u8] }
    }

    fn shape(&self) -> &Shape<P, V> {
        self.shape.as_ref()
    }

    fn material(&self) -> &Material<P, V> {
        self.material.as_ref()
    }

    fn surface(&self) -> Option<&Surface<P, V>> {
        None
    }
}
