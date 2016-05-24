extern crate nalgebra as na;
extern crate image;
extern crate std;
pub mod camera;

use std::collections::HashSet;
use std::time::Duration;
use std::any::TypeId;
use self::na::Point3;
use self::na::Vector3;
use self::image::Rgba;
use glium::glutin::VirtualKeyCode;
use universe::entity::HasId;
use universe::entity::Entity;
use universe::entity::Camera;
use universe::entity::Locatable;
use universe::entity::Rotatable;
use universe::entity::Updatable;
use universe::entity::Traceable;
use universe::entity::Shape;
use universe::entity::Material;
use universe::entity::Surface;
use SimulationContext;

pub type Entity3 = Entity<Point3<f32>, Vector3<f32>>;
pub type Camera3 = Camera<Point3<f32>, Vector3<f32>>;
pub type Traceable3 = Traceable<Point3<f32>, Vector3<f32>>;
pub type Locatable3 = Locatable<Point3<f32>>;
pub type Rotatable3 = Rotatable<Vector3<f32>>;
pub type Shape3 = Shape<Point3<f32>, Vector3<f32>>;
pub type Material3 = Material<Point3<f32>, Vector3<f32>>;
pub type Surface3 = Surface<Point3<f32>, Vector3<f32>>;

pub struct Entity3Impl {
    shape: Box<Shape3>,
    material: Option<Box<Material3>>,
    surface: Option<Box<Surface3>>,
}

impl Entity3Impl {
    pub fn new(shape: Box<Shape3>,
               material: Option<Box<Material3>>,
               surface: Option<Box<Surface3>>,
              ) -> Entity3Impl {
        Entity3Impl {
            shape: shape,
            material: material,
            surface: surface,
        }
    }
}

impl Entity<Point3<f32>, Vector3<f32>> for Entity3Impl {
    fn as_updatable_mut(&mut self) -> Option<&mut Updatable> {
        None
    }

    fn as_updatable(&self) -> Option<&Updatable> {
        None
    }

    fn as_traceable_mut(&mut self) -> Option<&mut Traceable<Point3<f32>, Vector3<f32>>> {
        None
    }

    fn as_traceable(&self) -> Option<&Traceable3> {
        None
    }
}

impl Traceable<Point3<f32>, Vector3<f32>> for Entity3Impl {
    fn trace(&self) -> Rgba<u8> {
        Rgba {
            data: [0u8, 0u8, 0u8, 0u8],
        }
    }

    fn shape(&self) -> &Shape3 {
        self.shape.as_ref()
    }

    fn material(&self) -> Option<&Material3> {
        self.material.as_ref().map(|x| &**x)
    }

    fn surface(&self) -> Option<&Surface3> {
        self.surface.as_ref().map(|x| &**x)
    }
}

pub struct Sphere {
    location: Point3<f32>,
    radius: f32,
}

impl Sphere {
    pub fn new(location: Point3<f32>, radius: f32) -> Sphere {
        Sphere {
            location: location,
            radius: radius,
        }
    }
}

impl HasId for Sphere {}

impl Shape<Point3<f32>, Vector3<f32>> for Sphere {
    fn get_normal_at(&self, point: &Point3<f32>) -> Vector3<f32> {
        let norm = *point - self.location;
        na::normalize(&norm)
    }

    fn is_point_inside(&self, point: &Point3<f32>) -> bool {
        na::distance_squared(&self.location, point) <= self.radius * self.radius
    }
}
