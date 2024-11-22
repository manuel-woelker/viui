use crate::resource::Resource;
use crate::result::ViuiResult;
use crate::types::Float;
use rustybuzz::{shape, Face, UnicodeBuffer};
use self_cell::self_cell;

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

self_cell!(
    struct FontData {
        owner: Box<[u8]>,
        #[covariant]
        dependent: FaceInfo,
    }
);

struct FaceInfo<'font> {
    face: Face<'font>,
}

pub struct TextMeasurer {
    font_data: FontData,
}

impl TextMeasurer {
    pub fn from_resource<R: Into<Resource>>(resource: R) -> ViuiResult<TextMeasurer> {
        let font_bytes = resource.into().as_bytes()?;
        let font_data = FontData::new(font_bytes, |font_bytes| FaceInfo {
            face: Face::from_slice(&font_bytes, 0).unwrap(),
        });
        Ok(TextMeasurer { font_data })
    }

    pub fn measure_text(&self, string: &str, size: Float) -> ViuiResult<TextMeasurement> {
        let face = &self.font_data.borrow_dependent().face;
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
    pub fn create_text_measurer() -> TextMeasurer {
        TextMeasurer::from_resource("assets/fonts/OpenSans-Regular.ttf").unwrap()
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
