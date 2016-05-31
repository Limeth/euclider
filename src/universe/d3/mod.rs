pub mod entity;

use std::collections::HashMap;
use std::any::TypeId;
use na::Point3;
use na::Vector3;
use na::PointAsVector;
use universe::entity::*;
use universe::d3::entity::camera::Camera3Impl;
use universe::Universe;
use universe::d3::entity::*;

pub struct Universe3D {
    camera: Box<Camera3>,
    entities: Vec<Box<Entity3>>,
    intersections: HashMap<(TypeId, TypeId), fn(&Point3<f32>,
                                                &Vector3<f32>,
                                                &Material<Point3<f32>>,
                                                &Shape<Point3<f32>>)
                                        -> Option<Intersection<Point3<f32>>>>,
}

impl Universe3D {
    pub fn new() -> Universe3D {
        Universe3D {
            camera: Box::new(Camera3Impl::new()),
            entities: Vec::new(),
            intersections: HashMap::new(),
        }
    }
}

impl Universe for Universe3D {
    type P = Point3<f32>;

    // fn trace(&self, location: &Point3<f32>, rotation: &Vector3<f32>) -> Option<Rgb<u8>> {
    //     Some(Rgb {
    //         data: [
    //             ((rotation.x + 1.0) * 255.0 / 2.0) as u8,
    //             ((rotation.y + 1.0) * 255.0 / 2.0) as u8,
    //             ((rotation.z + 1.0) * 255.0 / 2.0) as u8,
    //         ],
    //     })
    // }

    fn camera_mut(&mut self) -> &mut Camera<Point3<f32>> {
        &mut *self.camera
    }

    fn camera(&self) -> &Camera3 {
        &*self.camera
    }

    fn set_camera(&mut self, camera: Box<Camera3>) {
        self.camera = camera;
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

    fn intersectors_mut(&mut self)
                         -> &mut HashMap<(TypeId, TypeId),
                                         fn(&Self::P,
                                                     &<Self::P as PointAsVector>::Vector,
                                                     &Material<Self::P>,
                                                     &Shape<Self::P>)
                                                     -> Option<Intersection<Self::P>>> {
        &mut self.intersections
    }

    fn intersectors(&self)
                     -> &HashMap<(TypeId, TypeId),
                                 fn(&Self::P,
                                             &<Self::P as PointAsVector>::Vector,
                                             &Material<Self::P>,
                                             &Shape<Self::P>)
                                             -> Option<Intersection<Self::P>>> {
        &self.intersections
    }

    #[rustfmt_skip]
    fn set_intersectors(&mut self,
                         intersections: HashMap<(TypeId, TypeId),
                                                fn(&Self::P,
                                                            &<Self::P as PointAsVector>::Vector,
                                                            &Material<Self::P>,
                                                            &Shape<Self::P>)
                                                            -> Option<Intersection<Self::P>>>) {
        self.intersections = intersections;
    }
}
