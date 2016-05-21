extern crate nalgebra as na;
extern crate image;

use self::image::Rgb;
use self::na::*;
use universe::entity::camera::Camera;
use universe::Universe;
use universe::entity::Entity;

pub struct Universe3D {
    camera: Camera,
    entities: Vec<Box<Entity<Point3<f32>, Vector3<f32>>>>,
}

impl Universe3D {
    pub fn new() -> Universe3D {
        Universe3D {
            camera: Camera::new(),
            entities: Vec::new(),
        }
    }
}

impl Universe for Universe3D {
    type P = Point3<f32>;
    type V = Vector3<f32>;

    fn trace(&self, location: &Point3<f32>, rotation: &Vector3<f32>) -> Rgb<u8> {
        Rgb {
            data: [
                ((rotation.x + 1.0) * 255.0 / 2.0) as u8,
                ((rotation.y + 1.0) * 255.0 / 2.0) as u8,
                ((rotation.z + 1.0) * 255.0 / 2.0) as u8,
            ],
        }
    }

    fn camera_mut(&mut self) -> &mut Camera {
        &mut self.camera
    }

    fn camera(&self) -> &Camera {
        &self.camera
    }

    fn set_camera(&mut self, camera: &Camera) {
        self.camera = *camera;
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
}

