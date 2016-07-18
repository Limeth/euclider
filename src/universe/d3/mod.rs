pub mod entity;

use std::collections::HashMap;
use na::Point3;
use na::Vector3;
use universe::entity::Camera;
use universe::entity::shape::GeneralIntersectors;
use universe::entity::material::TransitionHandlers;
use universe::entity::surface::MappedTexture;
use universe::entity::surface::MappedTextureTransparent;
use universe::d3::entity::camera::Camera3Impl;
use universe::d3::entity::Camera3;
use universe::d3::entity::Entity3;
use universe::Universe;
use util::CustomFloat;

pub struct Universe3D<F: CustomFloat> {
    camera: Box<Camera3<F>>,
    entities: Vec<Box<Entity3<F>>>,
    intersections: GeneralIntersectors<F, Point3<F>, Vector3<F>>,
    transitions: TransitionHandlers<F, Point3<F>, Vector3<F>>,
    background: Box<MappedTexture<F, Point3<F>, Vector3<F>>>,
}

impl<F: CustomFloat> Universe3D<F> {
    pub fn new() -> Universe3D<F> {
        Universe3D {
            camera: Box::new(Camera3Impl::new()),
            entities: Vec::new(),
            intersections: HashMap::new(),
            transitions: HashMap::new(),
            background: Box::new(MappedTextureTransparent::new()),
        }
    }
}

impl<F: CustomFloat> Universe<F> for Universe3D<F> {
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
