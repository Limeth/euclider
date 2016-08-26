pub mod entity;

use std::sync::Arc;
use std::sync::RwLock;
use std::collections::HashMap;
use na::Point4;
use na::Vector4;
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

pub struct Universe4<F: CustomFloat> {
    pub camera: Arc<RwLock<Box<Camera4<F>>>>,
    pub entities: Vec<Box<Entity4<F>>>,
    pub intersections: GeneralIntersectors<F, Point4<F>, Vector4<F>>,
    pub background: Box<MappedTexture<F, Point4<F>, Vector4<F>>>,
}

impl<F: CustomFloat> Universe4<F> {
    pub fn construct(camera: Box<Camera4<F>>) -> Self {
        let mut intersectors: GeneralIntersectors<F, Point4<F>, Vector4<F>> = HashMap::new();

        intersectors.insert((Vacuum::id_static(), VoidShape::id_static()),
                            Box::new(intersect_void));
        intersectors.insert((Vacuum::id_static(), Sphere::<F, Point4<F>, Vector4<F>>::id_static()),
                            Box::new(Sphere::<F, Point4<F>, Vector4<F>>::intersect_linear));
        intersectors.insert((Vacuum::id_static(), Hyperplane::<F, Point4<F>, Vector4<F>>::id_static()),
                            Box::new(Hyperplane::<F, Point4<F>, Vector4<F>>::intersect_linear));
        intersectors.insert((Vacuum::id_static(), HalfSpace::<F, Point4<F>, Vector4<F>>::id_static()),
                            Box::new(HalfSpace::<F, Point4<F>, Vector4<F>>::intersect_linear));
        intersectors.insert((Vacuum::id_static(),
                     ComposableShape::<F, Point4<F>, Vector4<F>>::id_static()),
                    Box::new(ComposableShape::<F, Point4<F>, Vector4<F>>::intersect_linear));
        intersectors.insert((LinearSpace::<F, Point4<F>, Vector4<F>>::id_static(), VoidShape::id_static()),
                            Box::new(intersect_void));
        intersectors.insert((LinearSpace::<F, Point4<F>, Vector4<F>>::id_static(), Sphere::<F, Point4<F>, Vector4<F>>::id_static()),
                            Box::new(Sphere::<F, Point4<F>, Vector4<F>>::intersect_linear));
        intersectors.insert((LinearSpace::<F, Point4<F>, Vector4<F>>::id_static(), Hyperplane::<F, Point4<F>, Vector4<F>>::id_static()),
                            Box::new(Hyperplane::<F, Point4<F>, Vector4<F>>::intersect_linear));
        intersectors.insert((LinearSpace::<F, Point4<F>, Vector4<F>>::id_static(), HalfSpace::<F, Point4<F>, Vector4<F>>::id_static()),
                            Box::new(HalfSpace::<F, Point4<F>, Vector4<F>>::intersect_linear));
        intersectors.insert((LinearSpace::<F, Point4<F>, Vector4<F>>::id_static(),
                     ComposableShape::<F, Point4<F>, Vector4<F>>::id_static()),
                    Box::new(ComposableShape::<F, Point4<F>, Vector4<F>>::intersect_linear));

        Universe4 {
            camera: Arc::new(RwLock::new(camera)),
            entities: Vec::new(),
            intersections: intersectors,
            background: Box::new(MappedTextureTransparent::new()),
        }
    }
}

impl<F: CustomFloat> Universe<F> for Universe4<F> {
    type P = Point4<F>;
    type V = Vector4<F>;

    fn camera(&self) -> &RwLock<Box<Camera<F, Point4<F>, Vector4<F>>>> {
        &self.camera
    }

    fn entities_mut(&mut self) -> &mut Vec<Box<Entity4<F>>> {
        &mut self.entities
    }

    fn entities(&self) -> &Vec<Box<Entity4<F>>> {
        &self.entities
    }

    fn set_entities(&mut self, entities: Vec<Box<Entity4<F>>>) {
        self.entities = entities;
    }

    fn intersectors_mut(&mut self) -> &mut GeneralIntersectors<F, Point4<F>, Vector4<F>> {
        &mut self.intersections
    }

    fn intersectors(&self) -> &GeneralIntersectors<F, Point4<F>, Vector4<F>> {
        &self.intersections
    }

    fn set_intersectors(&mut self, intersections: GeneralIntersectors<F, Point4<F>, Vector4<F>>) {
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
