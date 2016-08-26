pub mod camera;
pub mod material;
pub mod shape;
pub mod surface;

use na::Point4;
use na::Vector4;
use util::CustomFloat;
use universe::entity::Entity;
use universe::entity::Camera;
use universe::entity::Traceable;
use universe::entity::Rotatable;
use universe::entity::Locatable;
use self::material::Material4;
use self::shape::Shape4;
use self::surface::Surface4;

pub type Entity4<F> = Entity<F, Point4<F>, Vector4<F>>;
pub type Camera4<F> = Camera<F, Point4<F>, Vector4<F>>;
pub type Traceable4<F> = Traceable<F, Point4<F>, Vector4<F>>;
pub type Locatable4<F> = Locatable<F, Point4<F>, Vector4<F>>;
pub type Rotatable4<F> = Rotatable<F, Point4<F>, Vector4<F>>;

pub struct Entity4Impl<F: CustomFloat> {
    shape: Box<Shape4<F>>,
    material: Box<Material4<F>>,
    surface: Option<Box<Surface4<F>>>,
}

unsafe impl<F: CustomFloat> Send for Entity4Impl<F> {}
unsafe impl<F: CustomFloat> Sync for Entity4Impl<F> {}

impl<F: CustomFloat> Entity4Impl<F> {
    pub fn new(shape: Box<Shape4<F>>,
               material: Box<Material4<F>>,
               surface: Option<Box<Surface4<F>>>)
               -> Self {
        Entity4Impl {
            shape: shape,
            material: material,
            surface: surface,
        }
    }

    pub fn new_with_surface(shape: Box<Shape4<F>>,
               material: Box<Material4<F>>,
               surface: Box<Surface4<F>>)
               -> Self {
        Self::new(shape, material, Some(surface))
    }

    pub fn new_without_surface(shape: Box<Shape4<F>>,
               material: Box<Material4<F>>)
               -> Self {
        Self::new(shape, material, None)
    }
}

impl<F: CustomFloat> Entity<F, Point4<F>, Vector4<F>> for Entity4Impl<F> {
    fn as_traceable_mut(&mut self) -> Option<&mut Traceable<F, Point4<F>, Vector4<F>>> {
        Some(self)
    }

    fn as_traceable(&self) -> Option<&Traceable4<F>> {
        Some(self)
    }
}

impl<F: CustomFloat> Traceable<F, Point4<F>, Vector4<F>> for Entity4Impl<F> {
    fn shape(&self) -> &Shape4<F> {
        self.shape.as_ref()
    }

    fn material(&self) -> &Material4<F> {
        self.material.as_ref()
    }

    fn surface(&self) -> Option<&Surface4<F>> {
        self.surface.as_ref().map(|x| &**x)
    }
}
