pub mod entity;

use ::F;
use std::sync::Arc;
use std::sync::RwLock;
use std::collections::HashMap;
use na;
use universe::entity::Camera;
use universe::entity::surface::MappedTexture;
use universe::entity::surface::MappedTextureTransparent;
use universe::d4::entity::Camera4;
use universe::d4::entity::Entity4;
use universe::entity::material::*;
use universe::entity::shape::*;
use universe::Universe;
use util::CustomFloat;
use util::HasId;
use core::ops::Deref;
use core::ops::DerefMut;

pub type Point4 = na::Point4<F>;
pub type Vector4 = na::Vector4<F>;

pub struct Universe4 {
    pub camera: Arc<RwLock<Box<Camera4>>>,
    pub entities: Vec<Box<Entity4>>,
    pub intersections: GeneralIntersectors<Point4, Vector4>,
    pub background: Box<MappedTexture<Point4, Vector4>>,
}

impl Universe4 {
    pub fn construct(camera: Box<Camera4>) -> Self {
        let mut intersectors: GeneralIntersectors<Point4, Vector4> = HashMap::new();

        intersectors.insert((Vacuum::id_static(), VoidShape::id_static()),
                            Box::new(intersect_void));
        intersectors.insert((Vacuum::id_static(), Sphere::<Point4, Vector4>::id_static()),
                            Box::new(Sphere::<Point4, Vector4>::intersect_linear));
        intersectors.insert((Vacuum::id_static(), Hyperplane::<Point4, Vector4>::id_static()),
                            Box::new(Hyperplane::<Point4, Vector4>::intersect_linear));
        intersectors.insert((Vacuum::id_static(), HalfSpace::<Point4, Vector4>::id_static()),
                            Box::new(HalfSpace::<Point4, Vector4>::intersect_linear));
        intersectors.insert((Vacuum::id_static(), Cylinder::<Point4, Vector4>::id_static()),
                            Box::new(Cylinder::<Point4, Vector4>::intersect_linear));
        intersectors.insert((Vacuum::id_static(),
                     ComposableShape::<Point4, Vector4>::id_static()),
                    Box::new(ComposableShape::<Point4, Vector4>::intersect_linear));
        intersectors.insert((LinearSpace::<Point4, Vector4>::id_static(), VoidShape::id_static()),
                            Box::new(intersect_void));
        intersectors.insert((LinearSpace::<Point4, Vector4>::id_static(), Sphere::<Point4, Vector4>::id_static()),
                            Box::new(Sphere::<Point4, Vector4>::intersect_linear));
        intersectors.insert((LinearSpace::<Point4, Vector4>::id_static(), Hyperplane::<Point4, Vector4>::id_static()),
                            Box::new(Hyperplane::<Point4, Vector4>::intersect_linear));
        intersectors.insert((LinearSpace::<Point4, Vector4>::id_static(), HalfSpace::<Point4, Vector4>::id_static()),
                            Box::new(HalfSpace::<Point4, Vector4>::intersect_linear));
        intersectors.insert((LinearSpace::<Point4, Vector4>::id_static(), Cylinder::<Point4, Vector4>::id_static()),
                            Box::new(Cylinder::<Point4, Vector4>::intersect_linear));
        intersectors.insert((LinearSpace::<Point4, Vector4>::id_static(),
                     ComposableShape::<Point4, Vector4>::id_static()),
                    Box::new(ComposableShape::<Point4, Vector4>::intersect_linear));

        Universe4 {
            camera: Arc::new(RwLock::new(camera)),
            entities: Vec::new(),
            intersections: intersectors,
            background: Box::new(MappedTextureTransparent::new()),
        }
    }
}

impl Universe for Universe4 {
    type P = Point4;
    type V = Vector4;

    fn camera(&self) -> &RwLock<Box<Camera<Point4, Vector4, Universe4>>> {
        &self.camera
    }

    fn entities_mut(&mut self) -> &mut Vec<Box<Entity4>> {
        &mut self.entities
    }

    fn entities(&self) -> &Vec<Box<Entity4>> {
        &self.entities
    }

    fn set_entities(&mut self, entities: Vec<Box<Entity4>>) {
        self.entities = entities;
    }

    fn intersectors_mut(&mut self) -> &mut GeneralIntersectors<Point4, Vector4> {
        &mut self.intersections
    }

    fn intersectors(&self) -> &GeneralIntersectors<Point4, Vector4> {
        &self.intersections
    }

    fn set_intersectors(&mut self, intersections: GeneralIntersectors<Point4, Vector4>) {
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
