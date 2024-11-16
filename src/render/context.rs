use crate::infrastructure::image_pool::ImagePool;
use crate::render::command::{ImageId, RenderCommand};
use crate::resource::Resource;
use crate::result::ViuiResult;

pub struct RenderContext<'a> {
    render_queue: Vec<RenderCommand>,
    image_pool: &'a mut ImagePool,
}

impl<'a> RenderContext<'a> {
    pub fn new(image_pool: &'a mut ImagePool) -> ViuiResult<Self> {
        let mut render_queue = vec![];
        // Add images to render queue
        // TODO: handle multiple backends
        for image_path in std::mem::take(&mut image_pool.images_to_load) {
            let image_id = image_pool.get_image_id(&image_path)?;
            render_queue.push(RenderCommand::LoadImage {
                image_id,
                resource: Resource::new(image_path),
            });
        }

        Ok(Self {
            render_queue,
            image_pool,
        })
    }
}
impl RenderContext<'_> {
    pub fn add_command(&mut self, command: RenderCommand) {
        self.render_queue.push(command);
    }

    pub fn get_image_id(&mut self, path: &str) -> ViuiResult<ImageId> {
        self.image_pool.get_image_id(path)
    }

    pub fn render_queue(self) -> Vec<RenderCommand> {
        self.render_queue
    }
}
