use crate::infrastructure::image_pool::ImagePool;
use crate::result::ViuiResult;
use crate::types::Size;

pub struct LayoutContext<'a> {
    image_pool: &'a mut ImagePool,
}

impl<'a> LayoutContext<'a> {
    pub fn new(image_pool: &'a mut ImagePool) -> LayoutContext<'a> {
        LayoutContext { image_pool }
    }
}

impl LayoutContext<'_> {
    pub fn get_image_size(&mut self, path: &str) -> ViuiResult<Size> {
        self.image_pool.get_image_size(path)
    }
}
