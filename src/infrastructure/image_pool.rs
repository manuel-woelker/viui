use crate::render::command::ImageId;
use crate::result::ViuiResult;
use crate::types::{Float, Size};
use image::ImageReader;
use std::collections::hash_map::Entry;
use std::collections::HashMap;
use std::io::BufReader;

#[derive(Debug)]
pub struct ImageEntry {
    image_id: ImageId,
    #[allow(dead_code)]
    path: String,
    size: Size,
}

#[derive(Debug, Default)]
pub struct ImagePool {
    pub path_to_image_id_map: HashMap<String, ImageEntry>,
    pub images_to_load: Vec<String>,
}

impl ImagePool {
    fn get_image_entry(&mut self, path: &str) -> ViuiResult<&ImageEntry> {
        let image_id = self.new_image_id();
        let entry = self.path_to_image_id_map.entry(path.to_string());
        match entry {
            Entry::Occupied(entry) => Ok(entry.into_mut()),
            Entry::Vacant(slot) => {
                let reader = ImageReader::new(BufReader::new(std::fs::File::open(path)?))
                    .with_guessed_format()?;
                let (width, height) = reader.into_dimensions()?;
                let size = Size::new(width as Float, height as Float);
                let entry = slot.insert(ImageEntry {
                    image_id,
                    path: path.to_string(),
                    size,
                });
                self.images_to_load.push(path.to_string());
                Ok(entry)
            }
        }
    }

    fn new_image_id(&self) -> ImageId {
        ImageId(self.path_to_image_id_map.len() as u64)
    }
    pub fn get_image_id(&mut self, path: &str) -> ViuiResult<ImageId> {
        Ok(self.get_image_entry(path)?.image_id)
    }

    pub fn get_image_size(&mut self, path: &str) -> ViuiResult<Size> {
        Ok(self.get_image_entry(path)?.size)
    }
}
