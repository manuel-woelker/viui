use crate::types::{Color, Float, Point, Rect};

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
}
