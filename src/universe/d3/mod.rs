pub mod entity;

use std::sync::Arc;
use std::sync::RwLock;
use std::sync::LockResult;
use std::sync::RwLockReadGuard;
use std::sync::RwLockWriteGuard;
use std::collections::HashMap;
use na::Point3;
use na::Vector3;
use universe::entity::Camera;
use universe::entity::surface::MappedTexture;
use universe::entity::surface::MappedTextureTransparent;
use universe::d3::entity::camera::Camera3Impl;
use universe::d3::entity::Camera3;
use universe::d3::entity::Entity3;
use universe::entity::material::*;
use universe::entity::shape::*;
use universe::d3::entity::shape::*;
use universe::Universe;
use util::CustomFloat;
use util::HasId;

pub struct Universe3<F: CustomFloat> {
    pub camera: Arc<RwLock<Box<Camera3<F>>>>,
    pub entities: Arc<RwLock<Vec<Box<Entity3<F>>>>>,
    pub intersections: GeneralIntersectors<F, Point3<F>, Vector3<F>>,
    pub background: Box<MappedTexture<F, Point3<F>, Vector3<F>>>,
}

impl<F: CustomFloat> Universe3<F> {
    pub fn default() -> Universe3<F> {
        let mut intersectors: GeneralIntersectors<F, Point3<F>, Vector3<F>> = HashMap::new();

        intersectors.insert((Vacuum::id_static(), VoidShape::id_static()),
                            Box::new(intersect_void));
        intersectors.insert((Vacuum::id_static(), Sphere::<F, Point3<F>, Vector3<F>>::id_static()),
                            Box::new(intersect_sphere3_linear));
        intersectors.insert((Vacuum::id_static(), Plane::<F, Point3<F>, Vector3<F>>::id_static()),
                            Box::new(Plane::<F, Point3<F>, Vector3<F>>::intersect_linear));
        intersectors.insert((Vacuum::id_static(), HalfSpace::<F, Point3<F>, Vector3<F>>::id_static()),
                            Box::new(HalfSpace::<F, Point3<F>, Vector3<F>>::intersect_linear));
        intersectors.insert((Vacuum::id_static(),
                     ComposableShape::<F, Point3<F>, Vector3<F>>::id_static()),
                    Box::new(ComposableShape::<F, Point3<F>, Vector3<F>>::intersect_linear));
        intersectors.insert((LinearSpace::<F, Point3<F>, Vector3<F>>::id_static(), VoidShape::id_static()),
                            Box::new(intersect_void));
        intersectors.insert((LinearSpace::<F, Point3<F>, Vector3<F>>::id_static(), Sphere::<F, Point3<F>, Vector3<F>>::id_static()),
                            Box::new(intersect_sphere3_linear));
        intersectors.insert((LinearSpace::<F, Point3<F>, Vector3<F>>::id_static(), Plane::<F, Point3<F>, Vector3<F>>::id_static()),
                            Box::new(Plane::<F, Point3<F>, Vector3<F>>::intersect_linear));
        intersectors.insert((LinearSpace::<F, Point3<F>, Vector3<F>>::id_static(), HalfSpace::<F, Point3<F>, Vector3<F>>::id_static()),
                            Box::new(HalfSpace::<F, Point3<F>, Vector3<F>>::intersect_linear));
        intersectors.insert((LinearSpace::<F, Point3<F>, Vector3<F>>::id_static(),
                     ComposableShape::<F, Point3<F>, Vector3<F>>::id_static()),
                    Box::new(ComposableShape::<F, Point3<F>, Vector3<F>>::intersect_linear));

        Universe3 {
            camera: Arc::new(RwLock::new(Box::new(Camera3Impl::new()))),
            entities: Arc::new(RwLock::new(Vec::new())),
            intersections: intersectors,
            background: Box::new(MappedTextureTransparent::new()),
        }
    }

    fn ref_of<T>(arg: &Arc<RwLock<T>>) -> RwLockReadGuard<T> {
        (*arg).read().expect("Tried to read a mutably borrowed field.")
    }

    fn ref_mut_of<T>(arg: &Arc<RwLock<T>>) -> RwLockWriteGuard<T> {
        (*arg).write().expect("Tried to mutably borrow a borrowed field.")
    }
}

impl<F: CustomFloat> Universe<F> for Universe3<F> {
    type P = Point3<F>;
    type V = Vector3<F>;

    fn camera_mut(&self) -> RwLockWriteGuard<Box<Camera<F, Point3<F>, Vector3<F>>>> {
        Self::ref_mut_of(&self.camera)
    }

    fn camera(&self) -> RwLockReadGuard<Box<Camera<F, Point3<F>, Vector3<F>>>> {
        Self::ref_of(&self.camera)
    }

    fn set_camera(&self, camera: Box<Camera3<F>>) {
        *Self::ref_mut_of(&self.camera) = camera;
    }

    fn entities_mut(&self) -> RwLockWriteGuard<Vec<Box<Entity3<F>>>> {
        Self::ref_mut_of(&self.entities)
    }

    fn entities(&self) -> RwLockReadGuard<Vec<Box<Entity3<F>>>> {
        Self::ref_of(&self.entities)
    }

    fn set_entities(&self, entities: Vec<Box<Entity3<F>>>) {
        *Self::ref_mut_of(&self.entities) = entities;
    }

    fn intersectors_mut(&mut self) -> &mut GeneralIntersectors<F, Point3<F>, Vector3<F>> {
        &mut self.intersections
    }

    fn intersectors(&self) -> &GeneralIntersectors<F, Point3<F>, Vector3<F>> {
        &self.intersections
    }

    fn set_intersectors(&mut self, intersections: GeneralIntersectors<F, Point3<F>, Vector3<F>>) {
        self.intersections = intersections;
    }

    fn background_mut(&mut self) -> &mut Box<MappedTexture<F, Self::P, Self::V>> {
        &mut self.background
    }

    fn background(&self) -> &Box<MappedTexture<F, Self::P, Self::V>> {
        &self.background
    }

    fn set_background(&mut self, background: Box<MappedTexture<F, Self::P, Self::V>>) {
        self.background = background;
    }
}
