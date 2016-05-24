extern crate nalgebra as na;
extern crate image;
pub mod entity;

use std::collections::HashMap;
use std::any::TypeId;
use na::Point3;
use na::Vector3;
use self::image::Rgb;
use universe::entity::Camera;
use universe::entity::Material;
use universe::entity::Shape;
use universe::d3::entity::camera::Camera3;
use universe::Universe;
use universe::entity::Entity;

pub struct Universe3D {
    camera: Box<Camera<Point3<f32>, Vector3<f32>>>,
    entities: Vec<Box<Entity<Point3<f32>, Vector3<f32>>>>,
    intersections: HashMap<(TypeId, TypeId), &'static Fn(Material<Point3<f32>, Vector3<f32>>, Shape<Point3<f32>, Vector3<f32>>) -> Option<Point3<f32>>>,
}

impl Universe3D {
    pub fn new() -> Universe3D {
        Universe3D {
            camera: Box::new(Camera3::new()),
            entities: Vec::new(),
            intersections: HashMap::new(),
        }
    }
}

impl Universe for Universe3D {
    type P = Point3<f32>;
    type V = Vector3<f32>;

    // fn trace(&self, location: &Point3<f32>, rotation: &Vector3<f32>) -> Option<Rgb<u8>> {
    //     Some(Rgb {
    //         data: [
    //             ((rotation.x + 1.0) * 255.0 / 2.0) as u8,
    //             ((rotation.y + 1.0) * 255.0 / 2.0) as u8,
    //             ((rotation.z + 1.0) * 255.0 / 2.0) as u8,
    //         ],
    //     })
    // }

    fn camera_mut(&mut self) -> &mut Camera<Point3<f32>, Vector3<f32>> {
        &mut *self.camera
    }

    fn camera(&self) -> &Camera<Point3<f32>, Vector3<f32>> {
        &*self.camera
    }

    fn set_camera(&mut self, camera: Box<Camera<Point3<f32>, Vector3<f32>>>) {
        self.camera = camera;
    }

    fn entities_mut(&mut self) -> &mut Vec<Box<Entity<Point3<f32>, Vector3<f32>>>> {
        &mut self.entities
    }

    fn entities(&self) -> &Vec<Box<Entity<Point3<f32>, Vector3<f32>>>> {
        &self.entities
    }

    fn set_entities(&mut self, entities: Vec<Box<Entity<Point3<f32>, Vector3<f32>>>>) {
        self.entities = entities;
    }

    fn intersections_mut(&mut self) -> &mut HashMap<(TypeId, TypeId), &'static Fn(Material<Self::P, Self::V>, Shape<Self::P, Self::V>) -> Option<Self::P>> {
        &mut self.intersections
    }

    fn intersections(&self) -> &HashMap<(TypeId, TypeId), &'static Fn(Material<Self::P, Self::V>, Shape<Self::P, Self::V>) -> Option<Self::P>> {
        &self.intersections
    }

    fn set_intersections(&mut self, intersections: HashMap<(TypeId, TypeId), &'static Fn(Material<Self::P, Self::V>, Shape<Self::P, Self::V>) -> Option<Self::P>>) {
        self.intersections = intersections;
    }
}

