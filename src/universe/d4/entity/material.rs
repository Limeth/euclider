use na::Point4;
use na::Vector4;
use universe::entity::material::Material;

pub type Material4<F> = Material<F, Point4<F>, Vector4<F>>;
