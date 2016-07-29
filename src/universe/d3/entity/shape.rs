use std::any::Any;
use std::any::TypeId;
use std::fmt;
use std::fmt::Debug;
use std::fmt::Display;
use std::iter;
use num::traits::NumCast;
use num::Zero;
use num::One;
use na;
use na::Cast;
use na::Point3;
use na::Vector3;
use util::CustomFloat;
use util::HasId;
use util::VecLazy;
use util::IterLazy;
use universe::entity::material::Material;
use universe::entity::material::Vacuum;
use universe::entity::shape::Shape;
use universe::entity::shape::Intersector;
use universe::entity::shape::IntersectionMarcher;
use universe::entity::shape::Intersection;
use universe::entity::shape::ComposableShape;
use universe::entity::shape::SetOperation;

pub type Shape3<F> = Shape<F, Point3<F>, Vector3<F>>;

#[allow(unused_variables)]
pub fn intersect_sphere3_in_vacuum<F: CustomFloat>
    (location: &Point3<F>,
     direction: &Vector3<F>,
     vacuum: &Material<F, Point3<F>, Vector3<F>>,
     sphere: &Shape<F, Point3<F>, Vector3<F>>,
     intersect: Intersector<F, Point3<F>, Vector3<F>>)
     -> Box<IntersectionMarcher<F, Point3<F>, Vector3<F>>> {
    use universe::entity::shape::Sphere;
    // Unsafe cast example:
    // let a = unsafe { &*(a as *const _ as *const Aimpl) };
    vacuum.as_any().downcast_ref::<Vacuum>().unwrap();
    let sphere: &Sphere<F, Point3<F>, Vector3<F>> =
        sphere.as_any().downcast_ref::<Sphere<F, Point3<F>, Vector3<F>>>().unwrap();

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
        return Box::new(iter::empty());
    }

    let d_sqrt = d.sqrt();
    let mut t_first: Option<F> = None;  // The smallest non-negative vector modifier
    let mut t_second: Option<F> = None;  // The second smallest non-negative vector modifier
    let t1: F = (-b - d_sqrt) / (<F as NumCast>::from(2.0).unwrap() * a);
    let t2: F = (-b + d_sqrt) / (<F as NumCast>::from(2.0).unwrap() * a);

    if t1 >= Cast::from(0.0) {
        t_first = Some(t1);

        if t2 >= Cast::from(0.0) {
            t_second = Some(t2);
        }
    } else if t2 >= Cast::from(0.0) {
        t_first = Some(t2);
    }

    if t_first.is_none() {
        return Box::new(iter::empty());  // Don't trace in the opposite direction
    }

    let mut closures: VecLazy<Intersection<F, Point3<F>, Vector3<F>>> = Vec::new();
    // Move the following variables inside the closures.
    // This lets the closures move outside the scope.
    let direction = *direction;
    let location = *location;
    let sphere_location = sphere.location;

    closures.push(Box::new(move || {
        let result_vector = direction * t_first.unwrap();
        let result_point = Point3::new(location.x + result_vector.x,
                                       location.y + result_vector.y,
                                       location.z + result_vector.z);

        let mut normal = result_point - sphere_location;
        normal = na::normalize(&normal);

        Some(Intersection::new(result_point,
                               direction,
                               normal,
                               na::distance_squared(&location, &result_point)))
    }));

    if t_second.is_some() {
        closures.push(Box::new(move || {
            let result_vector = direction * t_second.unwrap();
            let result_point = Point3::new(location.x + result_vector.x,
                                           location.y + result_vector.y,
                                           location.z + result_vector.z);

            let mut normal = result_point - sphere_location;
            normal = na::normalize(&normal);

            Some(Intersection::new(result_point,
                                   direction,
                                   normal,
                                   na::distance_squared(&location, &result_point)))
        }));
    }

    Box::new(IterLazy::new(closures))
}

pub struct Plane3<F: CustomFloat> {
    normal: Vector3<F>,
    constant: F,
}

impl<F: CustomFloat> Plane3<F> {
    pub fn new(normal: &Vector3<F>, constant: F) -> Plane3<F> {
        if na::distance_squared(&na::origin(), normal.as_point()) <= Cast::from(0.0) {
            panic!("Cannot have a normal with length of 0.");
        }

        Plane3 {
            normal: *normal,
            constant: constant,
        }
    }

    pub fn new_with_point(normal: &Vector3<F>, point: &Point3<F>) -> Plane3<F> {
        // D = -(A*x + B*y + C*z)
        let constant = -na::dot(normal, point.as_vector());

        Self::new(normal, constant)
    }

    pub fn new_with_vectors(vector_a: &Vector3<F>,
                            vector_b: &Vector3<F>,
                            point: &Point3<F>)
                            -> Plane3<F> {
        // A*x + B*y + C*z + D = 0
        let normal = na::cross(vector_a, vector_b);

        Self::new_with_point(&normal, point)
    }

    #[allow(dead_code)]
    pub fn new_with_equation(a: F, b: F, c: F, d: F) -> Plane3<F> {
        Self::new(&Vector3::new(a, b, c), d)
    }

    #[allow(unused_variables)]
    pub fn intersect_in_vacuum(location: &Point3<F>,
                               direction: &Vector3<F>,
                               vacuum: &Material<F, Point3<F>, Vector3<F>>,
                               shape: &Shape<F, Point3<F>, Vector3<F>>,
                               intersect: Intersector<F, Point3<F>, Vector3<F>>)
                               -> Box<IntersectionMarcher<F, Point3<F>, Vector3<F>>> {
        vacuum.as_any().downcast_ref::<Vacuum>().unwrap();
        let plane: &Plane3<F> = shape.as_any().downcast_ref::<Plane3<F>>().unwrap();

        // A*x + B*y + C*z + D = 0

        let t: F = -(na::dot(&plane.normal, location.as_vector()) + plane.constant) /
                   na::dot(&plane.normal, direction);

        if t < Cast::from(0.0) {
            return Box::new(iter::empty());
        }

        let result_vector = *direction * t;
        let result_point = Point3::new(location.x + result_vector.x,
                                       location.y + result_vector.y,
                                       location.z + result_vector.z);

        let normal = plane.normal;

        Box::new(iter::once(Intersection::new(result_point,
                                              *direction,
                                              normal,
                                              na::distance_squared(location, &result_point))))
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

impl<F: CustomFloat> Shape<F, Point3<F>, Vector3<F>> for Plane3<F> {
    #[allow(unused_variables)]
    fn is_point_inside(&self, point: &Point3<F>) -> bool {
        false
    }
}

impl<F: CustomFloat> Debug for Plane3<F> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f,
               "Plane3 [ normal: {:?}, constant: {:?} ]",
               self.normal,
               self.constant)
    }
}

impl<F: CustomFloat> Display for Plane3<F> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f,
               "Plane3 [ normal: {}, constant: {} ]",
               self.normal,
               self.constant)
    }
}

pub struct HalfSpace3<F: CustomFloat> {
    plane: Plane3<F>,
    signum: F,
}

impl<F: CustomFloat> HalfSpace3<F> {
    pub fn new(plane: Plane3<F>, mut signum: F) -> HalfSpace3<F> {
        signum /= signum.abs();

        HalfSpace3 {
            plane: plane,
            signum: signum,
        }
    }

    pub fn new_with_point(plane: Plane3<F>, point_inside: &Point3<F>) -> HalfSpace3<F> {
        let identifier: F = na::dot(&plane.normal, point_inside.as_vector()) + plane.constant;

        Self::new(plane, identifier)
    }

    pub fn intersect_in_vacuum(location: &Point3<F>,
                               direction: &Vector3<F>,
                               vacuum: &Material<F, Point3<F>, Vector3<F>>,
                               shape: &Shape<F, Point3<F>, Vector3<F>>,
                               intersect: Intersector<F, Point3<F>, Vector3<F>>)
                               -> Box<IntersectionMarcher<F, Point3<F>, Vector3<F>>> {
        vacuum.as_any().downcast_ref::<Vacuum>().unwrap();
        let halfspace: &HalfSpace3<F> = shape.as_any().downcast_ref::<HalfSpace3<F>>().unwrap();
        let intersection = Plane3::<F>::intersect_in_vacuum(location,
                                                            direction,
                                                            vacuum,
                                                            &halfspace.plane,
                                                            intersect)
            .next();

        // Works so far, not sure why
        if intersection.is_some() {
            let mut intersection = intersection.unwrap();
            intersection.normal *= -halfspace.signum;
            return Box::new(iter::once(intersection));
        }

        Box::new(iter::empty())
    }

    pub fn cuboid(center: Point3<F>, abc: Vector3<F>) -> ComposableShape<F, Point3<F>, Vector3<F>> {
        let half_abc: Vector3<F> = abc / <F as NumCast>::from(2.0).unwrap();
        let x = Vector3::new(<F as One>::one(), <F as Zero>::zero(), <F as Zero>::zero());
        let y = Vector3::new(<F as Zero>::zero(), <F as One>::one(), <F as Zero>::zero());
        let z = Vector3::new(<F as Zero>::zero(), <F as Zero>::zero(), <F as One>::one());
        ComposableShape::of(vec![HalfSpace3::new_with_point(Plane3::new_with_vectors(&y,
                                                                                 &z,
                                                                                 &na::translate(&(x *
                                                                                     half_abc),
                                                                                   &center),
                                                        ),
                                                        &center),
                                 HalfSpace3::new_with_point(Plane3::new_with_vectors(&y,
                                                                                 &z,
                                                                                 &na::translate(&-(x *
                                                                                      half_abc),
                                                                                   &center),
                                                        ),
                                                        &center),
                                 HalfSpace3::new_with_point(Plane3::new_with_vectors(&x,
                                                                                 &z,
                                                                                 &na::translate(&(y *
                                                                                     half_abc),
                                                                                   &center),
                                                        ),
                                                        &center),
                                 HalfSpace3::new_with_point(Plane3::new_with_vectors(&x,
                                                                                 &z,
                                                                                 &na::translate(&-(y *
                                                                                      half_abc),
                                                                                   &center),
                                                        ),
                                                        &center),
                                 HalfSpace3::new_with_point(Plane3::new_with_vectors(&x,
                                                                                 &y,
                                                                                 &na::translate(&(z *
                                                                                     half_abc),
                                                                                   &center),
                                                        ),
                                                        &center),
                                 HalfSpace3::new_with_point(Plane3::new_with_vectors(&x,
                                                                                 &y,
                                                                                 &na::translate(&-(z *
                                                                                      half_abc),
                                                                                   &center),
                                                        ),
                                                        &center)]
                                .into_iter(),
                            SetOperation::Intersection)
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

impl<F: CustomFloat> Shape<F, Point3<F>, Vector3<F>> for HalfSpace3<F> {
    fn is_point_inside(&self, point: &Point3<F>) -> bool {
        // A*x + B*y + C*z + D = 0
        // ~~~~~~~~~~~~~~~ dot
        let result: F = na::dot(&self.plane.normal, point.as_vector()) + self.plane.constant;

        self.signum == result.signum()
    }
}

impl<F: CustomFloat> Debug for HalfSpace3<F> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f,
               "HalfSpace3 [ plane: {:?}, signum: {:?} ]",
               self.plane,
               self.signum)
    }
}

impl<F: CustomFloat> Display for HalfSpace3<F> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f,
               "HalfSpace3 [ plane: {}, signum: {} ]",
               self.plane,
               self.signum)
    }
}
