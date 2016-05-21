extern crate nalgebra as na;
extern crate image;
extern crate std;

use std::collections::HashSet;
use std::time::Duration;
use self::na::Point3;
use self::na::Vector3;
use self::image::Rgba;
use glium::glutin::VirtualKeyCode;
use universe::entity::Entity;
use universe::entity::Locatable;
use universe::entity::Rotatable;
use universe::entity::Updatable;
use universe::entity::Traceable;
use universe::entity::Shape;
use universe::entity::Material;
use universe::entity::Surface;
use SimulationContext;

pub struct EntityImpl {
    shape: Box<Shape<Point3<f32>, Vector3<f32>>>,
    material: Option<Box<Material<Point3<f32>, Vector3<f32>>>>,
    surface: Option<Box<Surface<Point3<f32>, Vector3<f32>>>>,
}

impl EntityImpl {
    pub fn new(shape: Box<Shape<Point3<f32>, Vector3<f32>>>,
               material: Option<Box<Material<Point3<f32>, Vector3<f32>>>>,
               surface: Option<Box<Surface<Point3<f32>, Vector3<f32>>>>,
              ) -> EntityImpl {
        EntityImpl {
            shape: shape,
            material: material,
            surface: surface,
        }
    }
}

impl Entity<Point3<f32>, Vector3<f32>> for EntityImpl {
    fn as_updatable(&mut self) -> Option<&mut Updatable> {
        None
    }

    fn as_traceable(&mut self) -> Option<&mut Traceable<Point3<f32>, Vector3<f32>>> {
        None
    }
}

impl Traceable<Point3<f32>, Vector3<f32>> for EntityImpl {
    fn trace(&self) -> Rgba<u8> {
        Rgba {
            data: [0u8, 0u8, 0u8, 0u8],
        }
    }

    fn is_point_inside(&self, point: Point3<f32>) -> bool {
        self.shape.is_point_inside(point)
    }

    fn shape(&self) -> &Box<Shape<Point3<f32>, Vector3<f32>>> {
        &self.shape
    }

    fn material(&self) -> &Option<Box<Material<Point3<f32>, Vector3<f32>>>> {
        &self.material
    }

    fn surface(&self) -> &Option<Box<Surface<Point3<f32>, Vector3<f32>>>> {
        &self.surface
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

impl Shape<Point3<f32>, Vector3<f32>> for Sphere {
    fn get_normal_at(&self, point: Point3<f32>) -> Vector3<f32> {
        let norm = point - self.location;
        na::normalize(&norm)
    }

    fn is_point_inside(&self, point: Point3<f32>) -> bool {
        na::distance_squared(&self.location, &point) <= self.radius * self.radius
    }
}
