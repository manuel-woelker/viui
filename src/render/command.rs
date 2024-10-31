use crate::types::{Color, Float, Rect};

#[derive(Clone, Debug)]
pub enum RenderCommand {
    Save,
    Restore,
    SetStrokeColor(Color),
    SetFillColor(Color),
    FillRect {rect: Rect},
    FillRoundRect {rect: Rect, radius: Float},
    Translate {x: Float, y: Float},
    ResetTransform,
    DrawText(String),
}