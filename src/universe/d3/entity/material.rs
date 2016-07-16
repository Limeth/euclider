use na::Point3;
use na::Vector3;
use universe::entity::material::Material;

pub type Material3<F> = Material<F, Point3<F>, Vector3<F>>;
