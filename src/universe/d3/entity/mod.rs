pub mod camera;
pub mod material;
pub mod shape;
pub mod surface;

use universe::d3::Point3;
use universe::d3::Vector3;
use std::sync::Arc;
use util::CustomFloat;
use universe::d3::Universe3;
use universe::entity::Entity;
use universe::entity::Camera;
use universe::entity::Traceable;
use universe::entity::Rotatable;
use universe::entity::Locatable;
use self::material::Material3;
use self::shape::Shape3;
use self::surface::Surface3;

pub type Entity3 = Entity<Point3, Vector3>;
pub type Camera3 = Camera<Point3, Vector3, Universe3>;
pub type Traceable3 = Traceable<Point3, Vector3>;
pub type Locatable3 = Locatable<Point3, Vector3>;
pub type Rotatable3 = Rotatable<Point3, Vector3>;

pub struct Entity3Impl {
    shape: Arc<Shape3>,
    material: Arc<Material3>,
    surface: Option<Box<Surface3>>,
}

impl Entity3Impl {
    pub fn new(shape: Box<Shape3>,
               material: Box<Material3>,
               surface: Option<Box<Surface3>>)
               -> Self {
        Entity3Impl {
            shape: shape.into(),
            material: material.into(),
            surface: surface,
        }
    }

    pub fn new_with_surface(shape: Box<Shape3>,
               material: Box<Material3>,
               surface: Box<Surface3>)
               -> Self {
        Self::new(shape, material, Some(surface))
    }

    pub fn new_without_surface(shape: Box<Shape3>,
               material: Box<Material3>)
               -> Self {
        Self::new(shape, material, None)
    }
}

impl Entity<Point3, Vector3> for Entity3Impl {
    fn as_traceable_mut(&mut self) -> Option<&mut Traceable<Point3, Vector3>> {
        Some(self)
    }

    fn as_traceable(&self) -> Option<&Traceable3> {
        Some(self)
    }
}

impl Traceable<Point3, Vector3> for Entity3Impl {
    fn shape(&self) -> &Shape3 {
        self.shape.as_ref()
    }

    fn material(&self) -> &Material3 {
        self.material.as_ref()
    }

    fn surface(&self) -> Option<&Surface3> {
        self.surface.as_ref().map(|x| &**x)
    }
}
