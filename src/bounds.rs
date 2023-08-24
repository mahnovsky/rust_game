#![allow(dead_code)]
#![allow(unused_variables)]

// local coords
#[derive(Debug, Clone, Default)]
pub struct Bounds {
    x: f32,
    y: f32,
    width: f32,
    height: f32,
}

impl Bounds {
    pub fn new(x: f32, y: f32, width: f32, height: f32) -> Self {
        Self {
            x,
            y,
            width,
            height,
        }
    }

    pub fn with_center_position(x: f32, y: f32, width: f32, height: f32) -> Self {
        let half_w = width / 2_f32;
        let half_h = height / 2_f32;
        Self {
            x: x - half_w,
            y: y - half_h,
            width,
            height,
        }
    }

    pub fn set_position(&mut self, x: f32, y: f32) {
        self.x = x;
        self.y = y;
    }

    pub fn set_center_position(&mut self, x: f32, y: f32) {
        let (hw, hh) = self.get_half_size();
        self.x = x - hw;
        self.y = y - hh;
    }

    pub fn get_position(&self) -> (f32, f32) {
        (self.x, self.y)
    }

    pub fn get_size(&self) -> (f32, f32) {
        (self.width, self.height)
    }

    pub fn get_half_size(&self) -> (f32, f32) {
        (self.width / 2_f32, self.height / 2_f32)
    }

    pub fn get_corners(&self, corners: &mut [(f32, f32); 4]) {
        let offset = [(0., 0.), (1., 0.), (1., 1.), (0., 1.)];
        for i in 0..4 {
            corners[i] = (
                self.x + self.width * offset[i].0,
                self.y + self.height * offset[i].1,
            );
        }
    }

    fn cross_by_x(&self, x: f32) -> bool {
        x >= self.x && x <= (self.x + self.width)
    }

    fn cross_by_y(&self, y: f32) -> bool {
        y >= self.y && y <= (self.y + self.height)
    }

    fn get_hor_points(&self) -> (f32, f32) {
        (self.x, self.x + self.width)
    }

    fn get_vert_points(&self) -> (f32, f32) {
        (self.y, self.y + self.height)
    }

    pub fn has_collision(&self, other: &Bounds) -> bool {
        (self.cross_by_x(other.x) || other.cross_by_x(self.x))
            && (self.cross_by_y(other.y) || other.cross_by_y(self.y))
    }

    pub fn is_inside(&self, x: f32, y: f32) -> bool {
        x > self.x && x < (self.x + self.width) && y > self.y && y < (self.y + self.height)
    }

    pub fn is_inside_other(&self, other: &Bounds) -> bool {
        let (x0, x1) = self.get_hor_points();
        let (y0, y1) = self.get_vert_points();
        let (ox0, ox1) = other.get_hor_points();
        let (oy0, oy1) = other.get_vert_points();

        x0 > ox0 && x1 < ox1 && y0 > oy0 && y1 < oy1
    }
}
