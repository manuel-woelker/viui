use crate::infrastructure::font_pool::FontData;
use crate::result::ViuiResult;
use crate::types::Float;
use rustybuzz::{shape, UnicodeBuffer};

#[derive(Debug, Default, PartialEq)]
pub struct TextMeasurement {
    pub width: Float,
    pub height: Float,
}

impl TextMeasurement {
    pub fn new(width: Float, height: Float) -> Self {
        TextMeasurement { width, height }
    }
}

pub struct TextMeasurer<'a> {
    face: &'a rustybuzz::Face<'a>,
}

impl<'a> TextMeasurer<'a> {
    pub fn new(font_data: &'a FontData) -> Self {
        TextMeasurer {
            face: font_data.face(),
        }
    }

    pub fn measure_text(&self, string: &str, size: Float) -> ViuiResult<TextMeasurement> {
        let face = self.face;
        let mut buffer = UnicodeBuffer::new();
        buffer.push_str(&string);
        let glyphs = shape(face, &[], buffer);
        let mut width = 0i32;
        let upm = face.units_per_em() as Float;
        for glyph_position in glyphs.glyph_positions() {
            width += glyph_position.x_advance;
        }
        let scale_factor = size / upm;
        Ok(TextMeasurement {
            width: width as Float * scale_factor,
            height: face.height() as Float * scale_factor,
        })
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;

    // function to create text measurer
    fn measure_text(string: &str, size: Float) -> ViuiResult<TextMeasurement> {
        let font_data = FontCell::from_resource("assets/fonts/OpenSans-Regular.ttf")?;
        TextMeasurer {
            face: font_data.face(),
        }
        .measure_text(string, size)
    }

    #[test]
    fn test_empty() {
        let measurer = create_text_measurer();
        assert_eq!(
            measurer.measure_text("", 1.0).unwrap(),
            TextMeasurement::new(0.0, 1.3618164)
        );
    }

    #[test]
    fn test_simple() {
        let measurer = create_text_measurer();
        // measure simple string
        assert_eq!(
            measurer.measure_text("m", 16.0).unwrap(),
            TextMeasurement::new(14.8125, 21.789063)
        );
    }
}