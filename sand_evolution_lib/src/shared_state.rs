#[derive(Clone)]
pub struct SharedState {
    pub points: Vec<(cgmath::Point2<i32>, u8)>,
}

impl SharedState {
    pub fn new() -> Self {
        Self { points: vec![] }
    }

    pub fn set_pixel(&mut self, x: i32, y: i32, t: u8) {
        self.points.push((cgmath::Point2::<i32>::new(x, y), t));
    }
}
 