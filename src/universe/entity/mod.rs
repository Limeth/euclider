#[macro_use]
pub mod material;
#[macro_use]
pub mod shape;
pub mod surface;

use std::time::Duration;
use simulation::SimulationContext;
use universe::entity::shape::Shape;
use universe::entity::shape::VoidShape;
use universe::entity::material::Material;
use universe::entity::material::Vacuum;
use universe::entity::surface::Surface;
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

pub trait Camera<F: CustomFloat, P: CustomPoint<F, V>, V: CustomVector<F, P>>
    : Entity<F, P, V> {
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
    fn max_depth(&self) -> u32;
}

pub trait Updatable<F: CustomFloat, P: CustomPoint<F, V>, V: CustomVector<F, P>>
    : Entity<F, P, V> {
    fn update(&mut self, delta_time: &Duration, context: &SimulationContext);
}

pub trait Traceable<F: CustomFloat, P: CustomPoint<F, V>, V: CustomVector<F, P>>
    : Entity<F, P, V> {
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
    fn rotation_mut(&mut self) -> &mut V;
    fn rotation(&self) -> &V;
    fn set_rotation(&mut self, location: V);
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

impl<F: CustomFloat, P: CustomPoint<F, V>, V: CustomVector<F, P>> Traceable<F, P, V> for Void<F,
                                                                                              P,
                                                                                              V> {
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
