pub trait Universe {
    fn render(&self, time: f32);
    fn update(&mut self, delta_time: f32);
}

pub struct Universe3D {

}

impl Universe3D {
    pub fn new() -> Universe3D {
        Universe3D {

        }
    }
}

impl Universe for Universe3D {
    fn render(&self, time: f32) {

    }

    fn update(&mut self, delta_time: f32) {

    }
}
