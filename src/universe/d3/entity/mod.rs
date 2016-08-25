pub mod camera;
pub mod material;
pub mod shape;
pub mod surface;

use num::Zero;
use num::One;
use na::Point3;
use na::Vector3;
use util::CustomFloat;
use universe::entity::Entity;
use universe::entity::Camera;
use universe::entity::Traceable;
use universe::entity::Rotatable;
use universe::entity::Locatable;
use self::material::Material3;
use self::shape::Shape3;
use self::surface::Surface3;

pub type Entity3<F> = Entity<F, Point3<F>, Vector3<F>>;
pub type Camera3<F> = Camera<F, Point3<F>, Vector3<F>>;
pub type Traceable3<F> = Traceable<F, Point3<F>, Vector3<F>>;
pub type Locatable3<F> = Locatable<F, Point3<F>, Vector3<F>>;
pub type Rotatable3<F> = Rotatable<F, Point3<F>, Vector3<F>>;

pub struct Entity3Impl<F: CustomFloat> {
    shape: Box<Shape3<F>>,
    material: Box<Material3<F>>,
    surface: Option<Box<Surface3<F>>>,
}

unsafe impl<F: CustomFloat> Send for Entity3Impl<F> {}
unsafe impl<F: CustomFloat> Sync for Entity3Impl<F> {}

impl<F: CustomFloat> Entity3Impl<F> {
    pub fn new(shape: Box<Shape3<F>>,
               material: Box<Material3<F>>,
               surface: Option<Box<Surface3<F>>>)
               -> Self {
        Entity3Impl {
            shape: shape,
            material: material,
            surface: surface,
        }
    }

    pub fn new_with_surface(shape: Box<Shape3<F>>,
               material: Box<Material3<F>>,
               surface: Box<Surface3<F>>)
               -> Self {
        Self::new(shape, material, Some(surface))
    }

    pub fn new_without_surface(shape: Box<Shape3<F>>,
               material: Box<Material3<F>>)
               -> Self {
        Self::new(shape, material, None)
    }
}

impl<F: CustomFloat> Entity<F, Point3<F>, Vector3<F>> for Entity3Impl<F> {
    fn as_traceable_mut(&mut self) -> Option<&mut Traceable<F, Point3<F>, Vector3<F>>> {
        Some(self)
    }

    fn as_traceable(&self) -> Option<&Traceable3<F>> {
        Some(self)
    }
}

impl<F: CustomFloat> Traceable<F, Point3<F>, Vector3<F>> for Entity3Impl<F> {
    fn shape(&self) -> &Shape3<F> {
        self.shape.as_ref()
    }

    fn material(&self) -> &Material3<F> {
        self.material.as_ref()
    }

    fn surface(&self) -> Option<&Surface3<F>> {
        self.surface.as_ref().map(|x| &**x)
    }
}
