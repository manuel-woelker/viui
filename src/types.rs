use rgb::{RGBA8};

pub struct ScreenSpace;
pub type Color = RGBA8;
pub type Float = f32;
pub type Rect = euclid::Rect<Float, ScreenSpace>;
pub type Point = euclid::Point2D<Float, ScreenSpace>;
pub type Size = euclid::Size2D<Float, ScreenSpace>;