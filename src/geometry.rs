
/*
#[derive(Debug, Default)]
pub struct Point {
    pub x: f32,
    pub y: f32,
}

impl Point {
    pub fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }
}

#[derive(Debug, Default)]
pub struct Rect {
    pub upper_left: Point,
    pub lower_right: Point,
}


impl Rect {
    pub fn new(left: f32, top: f32, width: f32, height: f32) -> Self {
        Self::from_points(Point { x: left, y: top }, Point { x: left+width, y: top+height })
    }

    pub fn from_points(upper_left: Point, lower_right: Point) -> Self {
        Self { upper_left, lower_right }
    }

    pub fn top(&self) -> f32 {
        self.upper_left.y
    }

    pub fn left(&self) -> f32 {
        self.upper_left.x
    }

    pub fn width(&self) -> f32 {
        self.lower_right.x - self.upper_left.x
    }

    pub fn height(&self) -> f32 {
        self.lower_right.y - self.upper_left.y
    }

    pub fn contains(&self, point: &Point) -> bool {
        self.upper_left.x <= point.x && point.x <= self.lower_right.x && self.upper_left.y <= point.y && point.y <= self.lower_right.y
    }

}
*/
