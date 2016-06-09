pub mod entity;

use std::collections::HashMap;
use std::any::TypeId;
use na;
use na::PointAsVector;
use na::Point3;
use na::Vector3;
use image::Rgba;
use universe::entity::*;
use universe::d3::entity::camera::Camera3Impl;
use universe::Universe;
use universe::NalgebraOperations;
use universe::d3::entity::*;
use util::CustomFloat;

pub struct Universe3D<F: CustomFloat> {
    camera: Box<Camera3<F>>,
    entities: Vec<Box<Entity3<F>>>,
    operations: NalgebraOperations3,
    intersections: HashMap<(TypeId, TypeId),
                           fn(&Point3<F>,
                              &Vector3<F>,
                              &Material<F, Point3<F>>,
                              &Shape<F, Point3<F>>)
                              -> Option<Intersection<F, Point3<F>>>>,
    transitions: HashMap<(TypeId, TypeId),
                         fn(&Material<F, Point3<F>>,
                            &Material<F, Point3<F>>,
                            &TracingContext<F, Point3<F>>)
                            -> Rgba<u8>>,
}

impl<F: CustomFloat> Universe3D<F> {
    pub fn new() -> Universe3D<F> {
        Universe3D {
            camera: Box::new(Camera3Impl::new()),
            entities: Vec::new(),
            operations: NalgebraOperations3 {},
            intersections: HashMap::new(),
            transitions: HashMap::new(),
        }
    }
}

impl<F: CustomFloat> Universe<F> for Universe3D<F> {
    type P = Point3<F>;

    // fn trace(&self, location: &Point3<F>, rotation: &Vector3<F>) -> Option<Rgb<u8>> {
    //     Some(Rgb {
    //         data: [
    //             ((rotation.x + 1.0) * 255.0 / 2.0) as u8,
    //             ((rotation.y + 1.0) * 255.0 / 2.0) as u8,
    //             ((rotation.z + 1.0) * 255.0 / 2.0) as u8,
    //         ],
    //     })
    // }

    fn nalgebra_operations(&self) -> &NalgebraOperations<F, Point3<F>> {
        &self.operations
    }

    fn camera_mut(&mut self) -> &mut Camera<F, Point3<F>> {
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

    fn intersectors_mut(&mut self)
                         -> &mut HashMap<(TypeId, TypeId),
                                         fn(&Self::P,
                                                     &<Self::P as PointAsVector>::Vector,
                                                     &Material<F, Self::P>,
                                                     &Shape<F, Self::P>)
                                                     -> Option<Intersection<F, Self::P>>> {
        &mut self.intersections
    }

    fn intersectors(&self)
                    -> &HashMap<(TypeId, TypeId),
                                fn(&Self::P,
                                   &<Self::P as PointAsVector>::Vector,
                                   &Material<F, Self::P>,
                                   &Shape<F, Self::P>)
                                   -> Option<Intersection<F, Self::P>>> {
        &self.intersections
    }

    #[rustfmt_skip]
    fn set_intersectors(&mut self,
                         intersections: HashMap<(TypeId, TypeId),
                                                fn(&Self::P,
                                                            &<Self::P as PointAsVector>::Vector,
                                                            &Material<F, Self::P>,
                                                            &Shape<F, Self::P>)
                                                            -> Option<Intersection<F, Self::P>>>) {
        self.intersections = intersections;
    }

    fn transitions_mut(&mut self)
                       -> &mut HashMap<(TypeId, TypeId),
                                       fn(&Material<F, Self::P>,
                                          &Material<F, Self::P>,
                                          &TracingContext<F, Self::P>)
                                          -> Rgba<u8>> {
        &mut self.transitions
    }

    fn transitions(&self)
                   -> &HashMap<(TypeId, TypeId),
                               fn(&Material<F, Self::P>,
                                  &Material<F, Self::P>,
                                  &TracingContext<F, Self::P>)
                                  -> Rgba<u8>> {
        &self.transitions
    }
    fn set_transitions(&mut self,
                       transitions: HashMap<(TypeId, TypeId),
                                            fn(&Material<F, Self::P>,
                                               &Material<F, Self::P>,
                                               &TracingContext<F, Self::P>)
                                               -> Rgba<u8>>) {
        self.transitions = transitions
    }
}

#[derive(Copy, Clone)]
struct NalgebraOperations3;

impl<F: CustomFloat> NalgebraOperations<F, Point3<F>> for NalgebraOperations3 {
    fn to_point(&self, vector: &Vector3<F>) -> Point3<F> {
        vector.to_point()
    }

    fn dot(&self, first: &Vector3<F>, second: &Vector3<F>) -> F {
        na::dot(first, second)
    }

    fn angle_between(&self, first: &Vector3<F>, second: &Vector3<F>) -> F {
        na::angle_between(first, second)
    }
}
