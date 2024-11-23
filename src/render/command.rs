use crate::infrastructure::font_pool::FontIndex;
use crate::resource::Resource;
use crate::types::{Color, Float, Point, Rect, Size};

#[derive(Copy, Clone, Debug, Eq, Hash, PartialEq)]
pub struct ImageId(pub u64);

#[derive(Clone, Debug)]
pub enum RenderCommand {
    Save,
    Restore,
    SetStrokeColor(Color),
    SetStrokeWidth(Float),
    SetFillColor(Color),
    FillRect {
        rect: Rect,
    },
    FillRoundRect {
        rect: Rect,
        radius: Float,
    },
    Line {
        start: Point,
        end: Point,
    },
    Arc {
        center: Point,
        radius: Float,
        start_angle: Float,
        end_angle: Float,
    },
    Translate {
        x: Float,
        y: Float,
    },
    ResetTransform,
    DrawText(String),
    ClipRect(Rect),
    LoadImage {
        image_id: ImageId,
        resource: Resource,
    },
    LoadFont {
        font_idx: FontIndex,
        resource: Resource,
    },
    SetFont {
        font_idx: FontIndex,
    },
    SetWindowSize {
        size: Size,
    },
    DrawImage {
        image_id: ImageId,
    },
}
