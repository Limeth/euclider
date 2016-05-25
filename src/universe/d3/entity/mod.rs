extern crate nalgebra as na;
extern crate image;
extern crate std;
pub mod camera;

use std::any::Any;
use std::any::TypeId;
use self::na::Point3;
use self::na::Vector3;
use self::image::Rgba;
use universe::entity::*;

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
    material: Box<Material3>,
    surface: Option<Box<Surface3>>,
}

impl Entity3Impl {
    pub fn new(shape: Box<Shape3>,
               material: Box<Material3>,
               surface: Option<Box<Surface3>>)
               -> Entity3Impl {
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
        Rgba { data: [0u8, 0u8, 0u8, 0u8] }
    }

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

pub struct Sphere3 {
    location: Point3<f32>,
    radius: f32,
}

impl Sphere3 {
    pub fn new(location: Point3<f32>, radius: f32) -> Sphere3 {
        Sphere3 {
            location: location,
            radius: radius,
        }
    }
}

impl HasId for Sphere3 {
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


impl Shape<Point3<f32>, Vector3<f32>> for Sphere3 {
    fn get_normal_at(&self, point: &Point3<f32>) -> Vector3<f32> {
        let norm = *point - self.location;
        na::normalize(&norm)
    }

    fn is_point_inside(&self, point: &Point3<f32>) -> bool {
        na::distance_squared(&self.location, point) <= self.radius * self.radius
    }
}

pub fn intersect_void(location: &Point3<f32>, direction: &Vector3<f32>, vacuum: &Material<Point3<f32>, Vector3<f32>>, sphere: &Shape<Point3<f32>, Vector3<f32>>) -> Option<Point3<f32>> {
    None
}

pub fn intersect_sphere_in_vacuum(location: &Point3<f32>, direction: &Vector3<f32>, vacuum: &Material<Point3<f32>, Vector3<f32>>, sphere: &Shape<Point3<f32>, Vector3<f32>>) -> Option<Point3<f32>> {
    // Unsafe cast example:
    //let a = unsafe { &*(a as *const _ as *const Aimpl) };
    let vacuum: &Vacuum = vacuum.as_any().downcast_ref::<Vacuum>().unwrap();
    let sphere: &Sphere3 = sphere.as_any().downcast_ref::<Sphere3>().unwrap();

    let a = direction.x * direction.x + direction.y * direction.y + direction.z * direction.z;
    let b = direction.x * (2.0 * location.x - sphere.location.x)
            + direction.y * (2.0 * location.y - sphere.location.y)
            + direction.z * (2.0 * location.z - sphere.location.z);
    let c = location.x * (location.x - sphere.location.x)
            + location.y * (location.y - sphere.location.y)
            + location.z * (location.z - sphere.location.z)
            + sphere.location.x + sphere.location.y + sphere.location.z
            - sphere.radius * sphere.radius;

    // Discriminant = b^2 - 4*a*c
    let d = b * b - 4.0 * a * c;

    if d < 0.0 {
        return None;
    }

    let d_sqrt = d.sqrt();
    let t: f32; // The smallest non-negative vector modifier
    let t1 = (-b - d_sqrt) / (2.0 * a);

    if t1 >= 0.0 {
        t = t1;
    } else {
        let t2 = (-b + d_sqrt) / (2.0 * a);

        if t2 >= 0.0 {
            t = t2;
        } else {
            unreachable!();
        }
    }

    let result_vector = *direction * t;
    let result_point = Point3::new(location.x + result_vector.x, location.y + result_vector.y, location.z + result_vector.z);

    Some(result_point)
}
