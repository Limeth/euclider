pub mod entity;

use std::collections::HashMap;
use std::any::TypeId;
use na::PointAsVector;
use na::Point3;
use na::Vector3;
use palette::Rgba;
use universe::entity::*;
use universe::d3::entity::camera::Camera3Impl;
use universe::Universe;
use universe::d3::entity::*;
use util::CustomFloat;
use util::Provider;

pub struct Universe3D<F: CustomFloat> {
    camera: Box<Camera3<F>>,
    entities: Vec<Box<Entity3<F>>>,
    intersections: HashMap<(TypeId, TypeId),
                           fn(&Point3<F>,
                              &Vector3<F>,
                              &Material<F, Point3<F>, Vector3<F>>,
                              &Shape<F, Point3<F>, Vector3<F>>,
                              &Fn(
                                  &Material<F, Point3<F>, Vector3<F>>,
                                  &Shape<F, Point3<F>, Vector3<F>>
                              ) -> Option<Intersection<F, Point3<F>, Vector3<F>>>)
                              -> Provider<Intersection<F, Point3<F>, Vector3<F>>>>,
    transitions: HashMap<(TypeId, TypeId),
                         fn(&Material<F, Point3<F>, Vector3<F>>,
                            &Material<F, Point3<F>, Vector3<F>>,
                            &TracingContext<F, Point3<F>, Vector3<F>>)
                            -> Option<Rgba<F>>>,
}

impl<F: CustomFloat> Universe3D<F> {
    pub fn new() -> Universe3D<F> {
        Universe3D {
            camera: Box::new(Camera3Impl::new()),
            entities: Vec::new(),
            intersections: HashMap::new(),
            transitions: HashMap::new(),
        }
    }
}

impl<F: CustomFloat> Universe<F> for Universe3D<F> {
    type P = Point3<F>;
    type V = Vector3<F>;

    // fn trace(&self, location: &Point3<F>, rotation: &Vector3<F>) -> Option<Rgb<u8>> {
    //     Some(Rgb {
    //         data: [
    //             ((rotation.x + 1.0) * 255.0 / 2.0) as u8,
    //             ((rotation.y + 1.0) * 255.0 / 2.0) as u8,
    //             ((rotation.z + 1.0) * 255.0 / 2.0) as u8,
    //         ],
    //     })
    // }

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

    fn intersectors_mut(&mut self)
                         -> &mut HashMap<(TypeId, TypeId),
                                         fn(&Self::P,
                                                     &<Self::P as PointAsVector>::Vector,
                                                     &Material<F, Self::P, Self::V>,
                                                     &Shape<F, Self::P, Self::V>,
                                                     &Fn(
                                                         &Material<F, Self::P, Self::V>,
                                                         &Shape<F, Self::P, Self::V>
                                                     ) -> Option<Intersection<F, Self::P, Self::V>>)
                                                     -> Provider<Intersection<F, Self::P, Self::V>>> {
        &mut self.intersections
    }

    fn intersectors(&self)
                    -> &HashMap<(TypeId, TypeId),
                                fn(&Self::P,
                                   &<Self::P as PointAsVector>::Vector,
                                   &Material<F, Self::P, Self::V>,
                                   &Shape<F, Self::P, Self::V>,
                                   &Fn(
                                       &Material<F, Self::P, Self::V>,
                                       &Shape<F, Self::P, Self::V>
                                   ) -> Option<Intersection<F, Self::P, Self::V>>)
                                   -> Provider<Intersection<F, Self::P, Self::V>>> {
        &self.intersections
    }

    fn set_intersectors(&mut self,
                         intersections: HashMap<(TypeId, TypeId),
                                                fn(&Self::P,
                                                            &<Self::P as PointAsVector>::Vector,
                                                            &Material<F, Self::P, Self::V>,
                                                            &Shape<F, Self::P, Self::V>,
                                                            &Fn(
                                                                &Material<F, Self::P, Self::V>,
                                                                &Shape<F, Self::P, Self::V>
                                                            ) -> Option<Intersection<F, Self::P, Self::V>>)
                                                            -> Provider<Intersection<F, Self::P, Self::V>>>) {
        self.intersections = intersections;
    }

    fn transitions_mut(&mut self)
                       -> &mut HashMap<(TypeId, TypeId),
                                       fn(&Material<F, Self::P, Self::V>,
                                          &Material<F, Self::P, Self::V>,
                                          &TracingContext<F, Self::P, Self::V>)
                                          -> Option<Rgba<F>>> {
        &mut self.transitions
    }

    fn transitions(&self)
                   -> &HashMap<(TypeId, TypeId),
                               fn(&Material<F, Self::P, Self::V>,
                                  &Material<F, Self::P, Self::V>,
                                  &TracingContext<F, Self::P, Self::V>)
                                  -> Option<Rgba<F>>> {
        &self.transitions
    }
    fn set_transitions(&mut self,
                       transitions: HashMap<(TypeId, TypeId),
                                            fn(&Material<F, Self::P, Self::V>,
                                               &Material<F, Self::P, Self::V>,
                                               &TracingContext<F, Self::P, Self::V>)
                                               -> Option<Rgba<F>>>) {
        self.transitions = transitions
    }
}
