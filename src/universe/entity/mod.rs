#[macro_use]
pub mod material;
#[macro_use]
pub mod shape;
pub mod surface;

use ::F;
use std::time::Duration;
use std::sync::Arc;
use simulation::SimulationContext;
use universe::Universe;
use universe::entity::shape::Shape;
use universe::entity::shape::VoidShape;
use universe::entity::material::Material;
use universe::entity::material::Vacuum;
use universe::entity::surface::Surface;
use util::CustomFloat;
use util::CustomPoint;
use util::CustomVector;

pub trait Entity<P: CustomPoint<V>, V: CustomVector<P>>: Send + Sync
{
    fn as_traceable_mut(&mut self) -> Option<&mut Traceable<P, V>>;
    fn as_traceable(&self) -> Option<&Traceable<P, V>>;
}

pub trait Camera<P: CustomPoint<V>, V: CustomVector<P>, U: Universe<P=P, V=V>>
    : Entity<P, V> {
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
    fn update(&mut self, delta_time: &Duration, context: &SimulationContext, universe: &U);
}

pub trait Traceable<P: CustomPoint<V>, V: CustomVector<P>>
    : Entity<P, V> {
    fn shape(&self) -> &Shape<P, V>;
    fn material(&self) -> &Material<P, V>;
    fn surface(&self) -> Option<&Surface<P, V>>;
}

pub trait Locatable<P: CustomPoint<V>, V: CustomVector<P>> {
    fn location_mut(&mut self) -> &mut P;
    fn location(&self) -> &P;
    fn set_location(&mut self, location: P);
}

pub trait Rotatable<P: CustomPoint<V>, V: CustomVector<P>> {
    fn rotation_mut(&mut self) -> &mut V;
    fn rotation(&self) -> &V;
    fn set_rotation(&mut self, location: V);
}

pub struct Void<P: CustomPoint<V>, V: CustomVector<P>> {
    shape: Arc<VoidShape>,
    material: Arc<Material<P, V>>,
}

impl<P: CustomPoint<V>, V: CustomVector<P>> Void<P, V> {
    pub fn new(material: Box<Material<P, V>>) -> Void<P, V> {
        Void {
            shape: Arc::new(VoidShape::new()),
            material: material.into(),
        }
    }

    pub fn new_with_vacuum() -> Void<P, V> {
        Void {
            shape: Arc::new(VoidShape::new()),
            material: Arc::new(Vacuum::new()),
        }
    }
}

impl<P: CustomPoint<V>, V: CustomVector<P>> Entity<P, V> for Void<P, V> {
    fn as_traceable_mut(&mut self) -> Option<&mut Traceable<P, V>> {
        Some(self)
    }

    fn as_traceable(&self) -> Option<&Traceable<P, V>> {
        Some(self)
    }
}

impl<P: CustomPoint<V>, V: CustomVector<P>> Traceable<P, V> for Void<P, V> {
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
