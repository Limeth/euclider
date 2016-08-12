use std::iter;
use num::traits::NumCast;
use num::Zero;
use num::One;
use na;
use na::Cast;
use na::Point3;
use na::Vector3;
use util::CustomFloat;
use util::VecLazy;
use util::IterLazy;
use universe::entity::material::Material;
use universe::entity::shape::Shape;
use universe::entity::shape::Plane;
use universe::entity::shape::HalfSpace;
use universe::entity::shape::Intersector;
use universe::entity::shape::IntersectionMarcher;
use universe::entity::shape::Intersection;
use universe::entity::shape::ComposableShape;
use universe::entity::shape::SetOperation;
use universe::entity::shape::Sphere;

pub type Shape3<F> = Shape<F, Point3<F>, Vector3<F>>;

pub fn cuboid<F: CustomFloat>(center: Point3<F>, abc: Vector3<F>) -> ComposableShape<F, Point3<F>, Vector3<F>> {
    let half_abc: Vector3<F> = abc / <F as NumCast>::from(2.0).unwrap();
    let x = Vector3::new(<F as One>::one(), <F as Zero>::zero(), <F as Zero>::zero());
    let y = Vector3::new(<F as Zero>::zero(), <F as One>::one(), <F as Zero>::zero());
    let z = Vector3::new(<F as Zero>::zero(), <F as Zero>::zero(), <F as One>::one());
    let shapes: Vec<Box<Shape<F, Point3<F>, Vector3<F>>>> = vec![Box::new(HalfSpace::new_with_point(Plane::new_with_vectors(&y,
                                                                             &z,
                                                                             &na::translate(&(x *
                                                                                 half_abc),
                                                                               &center),
                                                    ),
                                                    &center)),
                             Box::new(HalfSpace::new_with_point(Plane::new_with_vectors(&y,
                                                                             &z,
                                                                             &na::translate(&-(x *
                                                                                  half_abc),
                                                                               &center),
                                                    ),
                                                    &center)),
                             Box::new(HalfSpace::new_with_point(Plane::new_with_vectors(&x,
                                                                             &z,
                                                                             &na::translate(&(y *
                                                                                 half_abc),
                                                                               &center),
                                                    ),
                                                    &center)),
                             Box::new(HalfSpace::new_with_point(Plane::new_with_vectors(&x,
                                                                             &z,
                                                                             &na::translate(&-(y *
                                                                                  half_abc),
                                                                               &center),
                                                    ),
                                                    &center)),
                             Box::new(HalfSpace::new_with_point(Plane::new_with_vectors(&x,
                                                                             &y,
                                                                             &na::translate(&(z *
                                                                                 half_abc),
                                                                               &center),
                                                    ),
                                                    &center)),
                             Box::new(HalfSpace::new_with_point(Plane::new_with_vectors(&x,
                                                                             &y,
                                                                             &na::translate(&-(z *
                                                                                  half_abc),
                                                                               &center),
                                                    ),
                                                    &center))];
    ComposableShape::of(shapes,
                        SetOperation::Intersection)
}

#[allow(unused_variables)]
pub fn intersect_sphere3_linear<F: CustomFloat>
    (location: &Point3<F>,
     direction: &Vector3<F>,
     vacuum: &Material<F, Point3<F>, Vector3<F>>,
     sphere: &Shape<F, Point3<F>, Vector3<F>>,
     intersect: Intersector<F, Point3<F>, Vector3<F>>)
     -> Box<IntersectionMarcher<F, Point3<F>, Vector3<F>>> {
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

    let t_first = t_first.unwrap();
    let mut closures: VecLazy<Intersection<F, Point3<F>, Vector3<F>>> = Vec::new();
    // Move the following variables inside the closures.
    // This lets the closures move outside the scope.
    let direction = *direction;
    let location = *location;
    let sphere_location = sphere.location;

    closures.push(Box::new(move || {
        let result_vector = direction * t_first;
        let result_point = Point3::new(location.x + result_vector.x,
                                       location.y + result_vector.y,
                                       location.z + result_vector.z);

        let mut normal = result_point - sphere_location;
        normal = na::normalize(&normal);

        Some(Intersection::new(result_point,
                               direction,
                               normal,
                               t_first))
    }));

    if let Some(t_second) = t_second {
        closures.push(Box::new(move || {
            let result_vector = direction * t_second;
            let result_point = Point3::new(location.x + result_vector.x,
                                           location.y + result_vector.y,
                                           location.z + result_vector.z);

            let mut normal = result_point - sphere_location;
            normal = na::normalize(&normal);

            Some(Intersection::new(result_point,
                                   direction,
                                   normal,
                                   t_second))
        }));
    }

    Box::new(IterLazy::new(closures))
}
