pub struct FontData {
    cell: FontCell,
    resource: Resource,
}

self_cell!(
    struct FontCell {
        owner: Box<[u8]>,
        #[covariant]
        dependent: FaceInfo,
    }
);

#[derive(Copy, Clone, Debug, Eq, Hash, PartialEq)]
pub struct FontIndex {
    index: usize,
}

impl FontIndex {
    pub fn new(index: usize) -> Self {
        FontIndex { index }
    }
}

impl FontData {
    pub fn from_resource<R: Into<Resource>>(resource: R) -> ViuiResult<Self> {
        let resource = resource.into();
        let font_bytes = resource.as_bytes()?;
        let font_cell = FontCell::new(font_bytes, |font_bytes| FaceInfo {
            face: Face::from_slice(&font_bytes, 0).unwrap(),
        });

        Ok(FontData {
            cell: font_cell,
            resource,
        })
    }

    pub fn face(&self) -> &Face {
        &self.cell.borrow_dependent().face
    }

    pub fn resource(&self) -> &Resource {
        &self.resource
    }
}

struct FaceInfo<'font> {
    face: Face<'font>,
}

use crate::infrastructure::measure_text::{TextMeasurement, TextMeasurer};
use crate::resource::Resource;
use crate::result::ViuiResult;
use crate::types::Float;
use rustybuzz::Face;
use self_cell::self_cell;

pub struct FontPool {
    fonts: Vec<FontData>,
}

impl FontPool {
    pub fn new() -> Self {
        Self { fonts: Vec::new() }
    }

    pub fn load_font<R: Into<Resource>>(&mut self, resource: R) -> ViuiResult<FontIndex> {
        let font_index = FontIndex::new(self.fonts.len());
        self.fonts.push(FontData::from_resource(resource)?);
        Ok(font_index)
    }

    pub fn measure_text(
        &self,
        font_index: FontIndex,
        text: &str,
        size: Float,
    ) -> ViuiResult<TextMeasurement> {
        let font = &self.fonts[font_index.index];
        TextMeasurer::new(font).measure_text(text, size)
    }

    pub fn maximum_font_index(&self) -> usize {
        self.fonts.len()
    }

    pub fn get_fonts_from(&self, offset: usize) -> impl Iterator<Item = (FontIndex, &FontData)> {
        self.fonts
            .iter()
            .enumerate()
            .skip(offset)
            .map(|(idx, font)| (FontIndex::new(idx), font))
    }
}
