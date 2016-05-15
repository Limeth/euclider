pub trait Universe {
    fn render(&self);
    fn update(&mut self, delta: f32);
}

pub struct GDLUniverse {

}

impl GDLUniverse {
    fn new() -> GDLUniverse {
        GDLUniverse {

        }
    }
}
