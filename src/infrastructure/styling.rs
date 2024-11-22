use crate::infrastructure::font_pool::FontIndex;
use crate::types::{Color, Float};

pub struct Styling {
    // Basic colors
    pub background_color: Color,
    pub border_color: Color,
    pub text_color: Color,
    pub highlight_color: Color,
    pub inactive_color: Color,

    // Button colors
    pub button_color: Color,
    pub button_hover_color: Color,
    pub button_pressed_color: Color,

    // Text style
    pub font_face: FontIndex,
    pub text_size: Float,
}

impl Styling {
    pub fn light() -> Self {
        Styling {
            background_color: Color::WHITE,
            border_color: Color::BLACK,
            text_color: Color::BLACK,
            highlight_color: Color::hsl(300.0, 0.8, 0.3).unwrap(),
            inactive_color: Color::gray(220),
            button_color: Color::gray(210),
            button_hover_color: Color::gray(230),
            button_pressed_color: Color::gray(240),
            font_face: FontIndex::new(0),
            text_size: 14.0,
        }
    }

    pub fn dark() -> Self {
        Styling {
            background_color: Color::gray(20),
            border_color: Color::gray(150),
            text_color: Color::gray(230),
            highlight_color: Color::hsl(300.0, 0.8, 0.3).unwrap(),
            inactive_color: Color::gray(80),
            button_color: Color::gray(40),
            button_hover_color: Color::gray(60),
            button_pressed_color: Color::gray(80),
            font_face: FontIndex::new(0),
            text_size: 14.0,
        }
    }
}
