pub mod entity;

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
    pub camera: Box<Camera3<F>>,
    pub entities: Vec<Box<Entity3<F>>>,
    pub intersections: GeneralIntersectors<F, Point3<F>, Vector3<F>>,
    pub transitions: TransitionHandlers<F, Point3<F>, Vector3<F>>,
    pub background: Box<MappedTexture<F, Point3<F>, Vector3<F>>>,
}

impl<F: CustomFloat> Universe3<F> {
    pub fn default() -> Universe3<F> {
        let mut intersectors: GeneralIntersectors<F, Point3<F>, Vector3<F>> = HashMap::new();

        intersectors.insert((Vacuum::id_static(), VoidShape::id_static()),
                            Box::new(intersect_void));
        intersectors.insert((Vacuum::id_static(), Sphere::<F, Point3<F>, Vector3<F>>::id_static()),
                            Box::new(intersect_sphere3_in_vacuum));
        intersectors.insert((Vacuum::id_static(), Plane3::<F>::id_static()),
                            Box::new(Plane3::<F>::intersect_in_vacuum));
        intersectors.insert((Vacuum::id_static(), HalfSpace3::<F>::id_static()),
                            Box::new(HalfSpace3::<F>::intersect_in_vacuum));
        intersectors.insert((Vacuum::id_static(),
                     ComposableShape::<F, Point3<F>, Vector3<F>>::id_static()),
                     Box::new(ComposableShape::<F, Point3<F>, Vector3<F>>::intersect_in_vacuum));

        let mut transitions: TransitionHandlers<F, Point3<F>, Vector3<F>> = HashMap::new();

        transitions.insert((Vacuum::id_static(), Vacuum::id_static()),
                           Box::new(transition_vacuum_vacuum));

        Universe3 {
            camera: Box::new(Camera3Impl::new()),
            entities: Vec::new(),
            intersections: intersectors,
            transitions: transitions,
            background: Box::new(MappedTextureTransparent::new()),
        }
    }
}

impl<F: CustomFloat> Universe<F> for Universe3<F> {
    type P = Point3<F>;
    type V = Vector3<F>;

    fn camera_mut(&mut self) -> &mut Camera<F, Point3<F>, Vector3<F>> {
        &mut *self.camera
    }

    fn camera(&self) -> &Camera3<F> {
        &*self.camera
    }

    fn set_camera(&mut self, camera: Box<Camera3<F>>) {
        self.camera = camera;
    }

    fn entities_mut(&mut self) -> &mut Vec<Box<Entity3<F>>> {
        &mut self.entities
    }

    fn entities(&self) -> &Vec<Box<Entity3<F>>> {
        &self.entities
    }

    fn set_entities(&mut self, entities: Vec<Box<Entity3<F>>>) {
        self.entities = entities;
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

    fn transitions_mut(&mut self) -> &mut TransitionHandlers<F, Self::P, Self::V> {
        &mut self.transitions
    }

    fn transitions(&self) -> &TransitionHandlers<F, Self::P, Self::V> {
        &self.transitions
    }

    fn set_transitions(&mut self, transitions: TransitionHandlers<F, Self::P, Self::V>) {
        self.transitions = transitions;
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
