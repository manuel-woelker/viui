use crate::render::command::ImageId;
use std::collections::HashMap;

#[derive(Debug, Default)]
pub struct ImagePool {
    pub path_to_image_id_map: HashMap<String, ImageId>,
}

impl ImagePool {
    pub fn new_image_id(&self) -> ImageId {
        ImageId(self.path_to_image_id_map.len() as u64)
    }
    pub fn get(&mut self, path: &str) -> Option<ImageId> {
        self.path_to_image_id_map.get(path).copied()
    }

    pub fn set(&mut self, path: &str, image_id: ImageId) {
        self.path_to_image_id_map.insert(path.to_string(), image_id);
    }
}
