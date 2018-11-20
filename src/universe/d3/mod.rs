pub mod entity;

use ::F;
use std::sync::Arc;
use std::sync::RwLock;
use std::collections::HashMap;
use na;
use universe::entity::Camera;
use universe::entity::surface::MappedTexture;
use universe::entity::surface::MappedTextureTransparent;
use universe::d3::entity::Camera3;
use universe::d3::entity::Entity3;
use universe::entity::material::*;
use universe::entity::shape::*;
use universe::Universe;
use util::CustomFloat;
use util::HasId;
use core::ops::Deref;
use core::ops::DerefMut;

pub type Point3 = na::Point3<F>;
pub type Vector3 = na::Vector3<F>;

pub struct Universe3 {
    pub camera: Arc<RwLock<Box<Camera3>>>,
    pub entities: Vec<Box<Entity3>>,
    pub intersections: GeneralIntersectors<Point3, Vector3>,
    pub background: Box<MappedTexture<Point3, Vector3>>,
}

impl Universe3 {
    pub fn construct(camera: Box<Camera3>) -> Self {
        let mut intersectors: GeneralIntersectors<Point3, Vector3> = HashMap::new();

        intersectors.insert((Vacuum::id_static(), VoidShape::id_static()),
                            Box::new(intersect_void));
        intersectors.insert((Vacuum::id_static(), Sphere::<Point3, Vector3>::id_static()),
                            Box::new(Sphere::<Point3, Vector3>::intersect_linear));
        intersectors.insert((Vacuum::id_static(), Hyperplane::<Point3, Vector3>::id_static()),
                            Box::new(Hyperplane::<Point3, Vector3>::intersect_linear));
        intersectors.insert((Vacuum::id_static(), HalfSpace::<Point3, Vector3>::id_static()),
                            Box::new(HalfSpace::<Point3, Vector3>::intersect_linear));
        intersectors.insert((Vacuum::id_static(), Cylinder::<Point3, Vector3>::id_static()),
                            Box::new(Cylinder::<Point3, Vector3>::intersect_linear));
        intersectors.insert((Vacuum::id_static(),
                     ComposableShape::<Point3, Vector3>::id_static()),
                    Box::new(ComposableShape::<Point3, Vector3>::intersect_linear));
        intersectors.insert((LinearSpace::<Point3, Vector3>::id_static(), VoidShape::id_static()),
                            Box::new(intersect_void));
        intersectors.insert((LinearSpace::<Point3, Vector3>::id_static(), Sphere::<Point3, Vector3>::id_static()),
                            Box::new(Sphere::<Point3, Vector3>::intersect_linear));
        intersectors.insert((LinearSpace::<Point3, Vector3>::id_static(), Hyperplane::<Point3, Vector3>::id_static()),
                            Box::new(Hyperplane::<Point3, Vector3>::intersect_linear));
        intersectors.insert((LinearSpace::<Point3, Vector3>::id_static(), HalfSpace::<Point3, Vector3>::id_static()),
                            Box::new(HalfSpace::<Point3, Vector3>::intersect_linear));
        intersectors.insert((LinearSpace::<Point3, Vector3>::id_static(), Cylinder::<Point3, Vector3>::id_static()),
                            Box::new(Cylinder::<Point3, Vector3>::intersect_linear));
        intersectors.insert((LinearSpace::<Point3, Vector3>::id_static(),
                     ComposableShape::<Point3, Vector3>::id_static()),
                    Box::new(ComposableShape::<Point3, Vector3>::intersect_linear));

        Universe3 {
            camera: Arc::new(RwLock::new(camera)),
            entities: Vec::new(),
            intersections: intersectors,
            background: Box::new(MappedTextureTransparent::new()),
        }
    }
}

impl Universe for Universe3 {
    type P = Point3;
    type V = Vector3;

    fn camera(&self) -> &RwLock<Box<Camera<Point3, Vector3, Universe3>>> {
        &self.camera
    }

    fn entities_mut(&mut self) -> &mut Vec<Box<Entity3>> {
        &mut self.entities
    }

    fn entities(&self) -> &Vec<Box<Entity3>> {
        &self.entities
    }

    fn set_entities(&mut self, entities: Vec<Box<Entity3>>) {
        self.entities = entities;
    }

    fn intersectors_mut(&mut self) -> &mut GeneralIntersectors<Point3, Vector3> {
        &mut self.intersections
    }

    fn intersectors(&self) -> &GeneralIntersectors<Point3, Vector3> {
        &self.intersections
    }

    fn set_intersectors(&mut self, intersections: GeneralIntersectors<Point3, Vector3>) {
        self.intersections = intersections;
    }

    fn background_mut(&mut self) -> &mut MappedTexture<Self::P, Self::V> {
        self.background.deref_mut()
    }

    fn background(&self) -> &MappedTexture<Self::P, Self::V> {
        self.background.deref()
    }

    fn set_background(&mut self, background: Box<MappedTexture<Self::P, Self::V>>) {
        self.background = background;
    }
}
