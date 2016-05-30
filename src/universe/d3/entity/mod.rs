pub mod camera;

use std::any::Any;
use std::any::TypeId;
use std::time::Duration;
use rand::Rng;
use rand::Rand;
use na;
use na::Point3;
use na::Vector3;
use image::Rgba;
use noise::perlin4;
use noise::Seed;
use palette;
use palette::Hsv;
use palette::RgbHue;
use universe::entity::*;

pub type Entity3 = Entity<Point3<f32>, Vector3<f32>>;
pub type Camera3 = Camera<Point3<f32>, Vector3<f32>>;
pub type Updatable3 = Updatable<Point3<f32>, Vector3<f32>>;
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

unsafe impl Sync for Entity3Impl {}

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
    fn as_updatable_mut(&mut self) -> Option<&mut Updatable<Point3<f32>, Vector3<f32>>> {
        None
    }

    fn as_updatable(&self) -> Option<&Updatable3> {
        None
    }

    fn as_traceable_mut(&mut self) -> Option<&mut Traceable<Point3<f32>, Vector3<f32>>> {
        Some(self)
    }

    fn as_traceable(&self) -> Option<&Traceable3> {
        Some(self)
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

pub struct Test3 {}

impl HasId for Test3 {
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

impl Shape<Point3<f32>, Vector3<f32>> for Test3 {
    fn get_normal_at(&self, point: &Point3<f32>) -> Vector3<f32> {
        Vector3::new(1.0, 0.0, 0.0)
    }

    fn is_point_inside(&self, point: &Point3<f32>) -> bool {
        false
    }
}

pub fn intersect_test(location: &Point3<f32>, direction: &Vector3<f32>, material: &Material<Point3<f32>, Vector3<f32>>, void: &Shape<Point3<f32>, Vector3<f32>>) -> Option<Intersection<Point3<f32>>> {
    let t = (1.0 - location.x) / direction.x;

    if t <= 0.0 {
        return None;
    }

    let result = Point3::new(location.x + t * direction.x, location.y + t * direction.y, location.z + t * direction.z);

    if result.y > 1.0 || result.y < 0.0 || result.z > 1.0 || result.z < 0.0 {
        return None;
    }

    Some(Intersection {
        point: result,
        distance_squared: na::distance_squared(location, &result),
    })
}

pub fn intersect_void(location: &Point3<f32>, direction: &Vector3<f32>, material: &Material<Point3<f32>, Vector3<f32>>, void: &Shape<Point3<f32>, Vector3<f32>>) -> Option<Intersection<Point3<f32>>> {
    void.as_any().downcast_ref::<VoidShape>().unwrap();
    None
}

pub fn intersect_sphere_in_vacuum(location: &Point3<f32>, direction: &Vector3<f32>, vacuum: &Material<Point3<f32>, Vector3<f32>>, sphere: &Shape<Point3<f32>, Vector3<f32>>) -> Option<Intersection<Point3<f32>>> {
    // Unsafe cast example:
    //let a = unsafe { &*(a as *const _ as *const Aimpl) };
    vacuum.as_any().downcast_ref::<Vacuum>().unwrap();
    let sphere: &Sphere3 = sphere.as_any().downcast_ref::<Sphere3>().unwrap();

    let rel_x = location.x - sphere.location.x;
    let rel_y = location.y - sphere.location.y;
    let rel_z = location.z - sphere.location.z;
    let a = direction.x * direction.x + direction.y * direction.y + direction.z * direction.z;
    let b = 2.0 * (
            direction.x * rel_x
            + direction.y * rel_y
            + direction.z * rel_z
        );
    let c = rel_x * rel_x
            + rel_y * rel_y
            + rel_z * rel_z
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
            return None; // Don't trace in the opposite direction
        }
    }

    let result_vector = *direction * t;
    let result_point = Point3::new(location.x + result_vector.x, location.y + result_vector.y, location.z + result_vector.z);

    Some(Intersection {
        point: result_point,
        distance_squared: na::distance_squared(location, &result_point),
    })
}

pub struct PerlinSurface3 {
    seed: Seed,
    size: f32,
    speed: f32,
}

impl HasId for PerlinSurface3 {
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

impl PerlinSurface3 {
    pub fn new(seed: u32, size: f32, speed: f32) -> PerlinSurface3 {
        PerlinSurface3 {
            seed: Seed::new(seed),
            size: size,
            speed: speed,
        }
    }

    pub fn rand<R: Rng>(rng: &mut R, size: f32, speed: f32) -> PerlinSurface3 {
        PerlinSurface3 {
            seed: Seed::rand(rng),
            size: size,
            speed: speed,
        }
    }
}

impl Surface<Point3<f32>, Vector3<f32>> for PerlinSurface3 {
    fn get_color<'a>(&self, context: TracingContext<'a, Point3<f32>, Vector3<f32>>) -> Rgba<u8> {
        let time_millis = (context.time.clone() * 1000).as_secs() as f32 / 1000.0;
        let location = [context.intersection.point.x / self.size, context.intersection.point.y / self.size, context.intersection.point.z / self.size, time_millis * self.speed];
        let value = perlin4(&self.seed, &location);
        Rgba {
            data: palette::Rgba::from(Hsv::new(RgbHue::from(value * 360.0), 1.0, 1.0)).to_pixel(),
        }
    }
}
