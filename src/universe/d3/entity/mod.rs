pub mod camera;

use std::any::Any;
use std::any::TypeId;
use std::marker::PhantomData;
use num::traits::NumCast;
use num::Zero;
use num::One;
use rand::Rng;
use rand::Rand;
use na;
use na::Cast;
use na::Point3;
use na::Vector3;
use image::Rgba;
use noise::perlin4;
use noise::Seed;
use palette;
use palette::Hsv;
use palette::RgbHue;
use universe::entity::*;
use util::CustomFloat;

#[allow(non_snake_case)]
pub fn AXIS_Z<F: CustomFloat>() -> Vector3<F> {
    Vector3 {
        x: <F as Zero>::zero(),
        y: <F as Zero>::zero(),
        z: <F as One>::one(),
    }
}

pub type Entity3<F> = Entity<F, Point3<F>>;
pub type Camera3<F> = Camera<F, Point3<F>>;
pub type Updatable3<F> = Updatable<F, Point3<F>>;
pub type Traceable3<F> = Traceable<F, Point3<F>>;
pub type Locatable3<F> = Locatable<F, Point3<F>>;
pub type Rotatable3<F> = Rotatable<F, Point3<F>>;
pub type Shape3<F> = Shape<F, Point3<F>>;
pub type Material3<F> = Material<F, Point3<F>>;
pub type Surface3<F> = Surface<F, Point3<F>>;

pub struct Entity3Impl<F: CustomFloat> {
    shape: Box<Shape3<F>>,
    material: Box<Material3<F>>,
    surface: Option<Box<Surface3<F>>>,
}

unsafe impl<F: CustomFloat> Sync for Entity3Impl<F> {}

impl<F: CustomFloat> Entity3Impl<F> {
    pub fn new(shape: Box<Shape3<F>>,
               material: Box<Material3<F>>,
               surface: Option<Box<Surface3<F>>>)
               -> Entity3Impl<F> {
        Entity3Impl {
            shape: shape,
            material: material,
            surface: surface,
        }
    }
}

impl<F: CustomFloat> Entity<F, Point3<F>> for Entity3Impl<F> {
    fn as_updatable_mut(&mut self) -> Option<&mut Updatable<F, Point3<F>>> {
        None
    }

    fn as_updatable(&self) -> Option<&Updatable3<F>> {
        None
    }

    fn as_traceable_mut(&mut self) -> Option<&mut Traceable<F, Point3<F>>> {
        Some(self)
    }

    fn as_traceable(&self) -> Option<&Traceable3<F>> {
        Some(self)
    }
}

impl<F: CustomFloat> Traceable<F, Point3<F>> for Entity3Impl<F> {
    fn trace(&self) -> Rgba<u8> {
        Rgba { data: [0u8, 0u8, 0u8, 0u8] }
    }

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

pub struct Sphere3<F: CustomFloat> {
    location: Point3<F>,
    radius: F,
}

impl<F: CustomFloat> Sphere3<F> {
    pub fn new(location: Point3<F>, radius: F) -> Sphere3<F> {
        Sphere3 {
            location: location,
            radius: radius,
        }
    }
}

impl<F: CustomFloat> HasId for Sphere3<F> {
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

impl<F: CustomFloat> Shape<F, Point3<F>> for Sphere3<F> {
    fn get_normal_at(&self, point: &Point3<F>) -> Vector3<F> {
        let norm = *point - self.location;
        na::normalize(&norm)
    }

    fn is_point_inside(&self, point: &Point3<F>) -> bool {
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

impl<F: CustomFloat> Shape<F, Point3<F>> for Test3 {
    fn get_normal_at(&self, point: &Point3<F>) -> Vector3<F> {
        Vector3::new(<F as One>::one(), <F as Zero>::zero(), <F as Zero>::zero())
    }

    fn is_point_inside(&self, point: &Point3<F>) -> bool {
        false
    }
}

pub fn intersect_test<F: CustomFloat>(location: &Point3<F>,
                                      direction: &Vector3<F>,
                                      material: &Material<F, Point3<F>>,
                                      void: &Shape<F, Point3<F>>)
                                      -> Option<Intersection<F, Point3<F>>> {
    let t: F = (<F as One>::one() - location.x) / direction.x;

    if t <= <F as Zero>::zero() {
        return None;
    }

    let result = Point3::new(location.x + t * direction.x,
                             location.y + t * direction.y,
                             location.z + t * direction.z);

    if result.y > <F as One>::one() || result.y < <F as Zero>::zero() ||
       result.z > <F as One>::one() || result.z < <F as Zero>::zero() {
        return None;
    }

    Some(Intersection {
        location: result,
        direction: *direction,
        distance_squared: na::distance_squared(location, &result),
        float_precision: PhantomData,
    })
}

pub fn intersect_void<F: CustomFloat>(location: &Point3<F>,
                                      direction: &Vector3<F>,
                                      material: &Material<F, Point3<F>>,
                                      void: &Shape<F, Point3<F>>)
                                      -> Option<Intersection<F, Point3<F>>> {
    void.as_any().downcast_ref::<VoidShape>().unwrap();
    None
}

pub fn intersect_sphere_in_vacuum<F: CustomFloat>(location: &Point3<F>,
                                                  direction: &Vector3<F>,
                                                  vacuum: &Material<F, Point3<F>>,
                                                  sphere: &Shape<F, Point3<F>>)
                                                  -> Option<Intersection<F, Point3<F>>> {
    // Unsafe cast example:
    // let a = unsafe { &*(a as *const _ as *const Aimpl) };
    vacuum.as_any().downcast_ref::<Vacuum>().unwrap();
    let sphere: &Sphere3<F> = sphere.as_any().downcast_ref::<Sphere3<F>>().unwrap();

    let rel_x: F = location.x - sphere.location.x;
    let rel_y: F = location.y - sphere.location.y;
    let rel_z: F = location.z - sphere.location.z;
    let a: F = direction.x * direction.x + direction.y * direction.y + direction.z * direction.z;
    let b: F = <F as NumCast>::from(2.0).unwrap() *
               (direction.x * rel_x + direction.y * rel_y + direction.z * rel_z);
    let c: F = rel_x * rel_x + rel_y * rel_y + rel_z * rel_z - sphere.radius * sphere.radius;

    // Discriminant = b^2 - 4*a*c
    let d: F = b * b - <F as NumCast>::from(4.0).unwrap() * a * c;

    if d < Cast::from(0.0) {
        return None;
    }

    let d_sqrt = d.sqrt();
    let t: F; // The smallest non-negative vector modifier
    let t1 = (-b - d_sqrt) / (<F as NumCast>::from(2.0).unwrap() * a);

    if t1 >= Cast::from(0.0) {
        t = t1;
    } else {
        let t2 = (-b + d_sqrt) / (<F as NumCast>::from(2.0).unwrap() * a);

        if t2 >= Cast::from(0.0) {
            t = t2;
        } else {
            return None; // Don't trace in the opposite direction
        }
    }

    let result_vector = *direction * t;
    let result_point = Point3::new(location.x + result_vector.x,
                                   location.y + result_vector.y,
                                   location.z + result_vector.z);

    Some(Intersection {
        location: result_point,
        direction: *direction,
        distance_squared: na::distance_squared(location, &result_point),
        float_precision: PhantomData,
    })
}

pub fn intersect_plane_in_vacuum<F: CustomFloat>(location: &Point3<F>,
                                                 direction: &Vector3<F>,
                                                 vacuum: &Material<F, Point3<F>>,
                                                 shape: &Shape<F, Point3<F>>)
                                                 -> Option<Intersection<F, Point3<F>>> {
    vacuum.as_any().downcast_ref::<Vacuum>().unwrap();
    let plane: &Plane3<F> = shape.as_any().downcast_ref::<Plane3<F>>().unwrap();

    // A*x + B*y + C*z + D = 0

    let t: F = -(na::dot(&plane.normal, location.as_vector()) + plane.constant) /
               na::dot(&plane.normal, direction);

    if t < Cast::from(0.0) {
        return None;
    }

    let result_vector = *direction * t;
    let result_point = Point3::new(location.x + result_vector.x,
                                   location.y + result_vector.y,
                                   location.z + result_vector.z);

    Some(Intersection {
        location: result_point,
        direction: *direction,
        distance_squared: na::distance_squared(location, &result_point),
        float_precision: PhantomData,
    })
}

pub fn intersect_halfspace_in_vacuum<F: CustomFloat>(location: &Point3<F>,
                                                     direction: &Vector3<F>,
                                                     vacuum: &Material<F, Point3<F>>,
                                                     shape: &Shape<F, Point3<F>>)
                                                     -> Option<Intersection<F, Point3<F>>> {
    vacuum.as_any().downcast_ref::<Vacuum>().unwrap();
    let halfspace: &HalfSpace3<F> = shape.as_any().downcast_ref::<HalfSpace3<F>>().unwrap();

    intersect_plane_in_vacuum(location, direction, vacuum, &halfspace.plane)
}

pub struct PerlinSurface3<F: CustomFloat> {
    seed: Seed,
    size: F,
    speed: F,
}

impl<F: CustomFloat> HasId for PerlinSurface3<F> {
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

impl<F: CustomFloat> PerlinSurface3<F> {
    pub fn new(seed: u32, size: F, speed: F) -> PerlinSurface3<F> {
        PerlinSurface3 {
            seed: Seed::new(seed),
            size: size,
            speed: speed,
        }
    }

    pub fn rand<R: Rng>(rng: &mut R, size: F, speed: F) -> PerlinSurface3<F> {
        PerlinSurface3 {
            seed: Seed::rand(rng),
            size: size,
            speed: speed,
        }
    }
}

impl<F: CustomFloat> Surface<F, Point3<F>> for PerlinSurface3<F> {
    fn get_color<'a>(&self, context: TracingContext<'a, F, Point3<F>>) -> Rgba<u8> {
        let time_millis: F = Cast::from((context.time.clone() * 1000).as_secs() as f64 / 1000.0);
        let location = [context.intersection.location.x / self.size,
                        context.intersection.location.y / self.size,
                        context.intersection.location.z / self.size,
                        time_millis * self.speed];
        let value = perlin4(&self.seed, &location);
        Rgba {
            data: palette::Rgba::from(Hsv::new(RgbHue::from(value * Cast::from(360.0)),
                                               Cast::from(1.0),
                                               Cast::from(1.0)))
                .to_pixel(),
        }
    }
}

pub struct Plane3<F: CustomFloat> {
    normal: Vector3<F>,
    constant: F,
}

impl<F: CustomFloat> Plane3<F> {
    pub fn new(point: &Point3<F>, vector_a: &Vector3<F>, vector_b: &Vector3<F>) -> Plane3<F> {
        // A*x + B*y + C*z + D = 0
        let normal = na::cross(vector_a, vector_b);

        // D = -(A*x + B*y + C*z)
        let constant = -na::dot(&normal, point.as_vector());

        Self::from_normal(&normal, constant)
    }

    pub fn from_normal(normal: &Vector3<F>, constant: F) -> Plane3<F> {
        Plane3 {
            normal: *normal,
            constant: constant,
        }
    }

    pub fn from_equation(a: F, b: F, c: F, d: F) -> Plane3<F> {
        Self::from_normal(&Vector3::new(a, b, c), d)
    }
}

impl<F: CustomFloat> HasId for Plane3<F> {
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

impl<F: CustomFloat> Shape<F, Point3<F>> for Plane3<F> {
    fn get_normal_at(&self, point: &Point3<F>) -> Vector3<F> {
        self.normal
    }

    fn is_point_inside(&self, point: &Point3<F>) -> bool {
        false
    }
}

pub struct HalfSpace3<F: CustomFloat> {
    plane: Plane3<F>,
    point_inside: Point3<F>,
}

impl<F: CustomFloat> HalfSpace3<F> {
    pub fn new(plane: Plane3<F>, point_inside: &Point3<F>) -> HalfSpace3<F> {
        HalfSpace3 {
            plane: plane,
            point_inside: *point_inside,
        }
    }
}

impl<F: CustomFloat> HasId for HalfSpace3<F> {
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

impl<F: CustomFloat> Shape<F, Point3<F>> for HalfSpace3<F> {
    fn get_normal_at(&self, point: &Point3<F>) -> Vector3<F> {
        self.plane.get_normal_at(point)
    }

    fn is_point_inside(&self, point: &Point3<F>) -> bool {
        // A*x + B*y + C*z + D = 0
        // ~~~~~~~~~~~~~~~ dot
        let identifier: F = na::dot(&self.plane.normal, point.as_vector()) + self.plane.constant;
        let result: F = na::dot(&self.plane.normal, point.as_vector()) + self.plane.constant;

        identifier.signum() == result.signum()
    }
}
