pub mod camera;
pub mod material;
pub mod shape;
pub mod surface;

use ::F;
use universe::d4::Point4;
use universe::d4::Vector4;
use std::sync::Arc;
use util::CustomFloat;
use universe::d4::Universe4;
use universe::entity::Entity;
use universe::entity::Camera;
use universe::entity::Traceable;
use universe::entity::Rotatable;
use universe::entity::Locatable;
use self::material::Material4;
use self::shape::Shape4;
use self::surface::Surface4;

pub type Entity4 = Entity<Point4, Vector4>;
pub type Camera4 = Camera<Point4, Vector4, Universe4>;
pub type Traceable4 = Traceable<Point4, Vector4>;
pub type Locatable4 = Locatable<Point4, Vector4>;
pub type Rotatable4 = Rotatable<Point4, Vector4>;

pub struct Entity4Impl {
    shape: Arc<Shape4>,
    material: Arc<Material4>,
    surface: Option<Arc<Surface4>>,
}

impl Entity4Impl {
    pub fn new(shape: Box<Shape4>,
               material: Box<Material4>,
               surface: Option<Box<Surface4>>)
               -> Self {
        Entity4Impl {
            shape: shape.into(),
            material: material.into(),
            surface: surface.map(|surface| surface.into()),
        }
    }

    pub fn new_with_surface(shape: Box<Shape4>,
               material: Box<Material4>,
               surface: Box<Surface4>)
               -> Self {
        Self::new(shape, material, Some(surface))
    }

    pub fn new_without_surface(shape: Box<Shape4>,
               material: Box<Material4>)
               -> Self {
        Self::new(shape, material, None)
    }
}

impl Entity<Point4, Vector4> for Entity4Impl {
    fn as_traceable_mut(&mut self) -> Option<&mut Traceable<Point4, Vector4>> {
        Some(self)
    }

    fn as_traceable(&self) -> Option<&Traceable4> {
        Some(self)
    }
}

impl Traceable<Point4, Vector4> for Entity4Impl {
    fn shape(&self) -> &Shape4 {
        self.shape.as_ref()
    }

    fn material(&self) -> &Material4 {
        self.material.as_ref()
    }

    fn surface(&self) -> Option<&Surface4> {
        self.surface.as_ref().map(|x| &**x)
    }
}
